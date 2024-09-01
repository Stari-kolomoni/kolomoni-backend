use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
};

use chrono::{DateTime, SecondsFormat, Utc};
use kolomoni_migrations_core::{
    configuration::MigrationConfiguration,
    identifier::MigrationIdentifier,
};
use miette::{miette, Context, IntoDiagnostic, Result};

use crate::cli::{GenerateCommandArguments, GeneratedScriptType};


fn write_str_to_new_file(file_path: &Path, contents: &str) -> Result<()> {
    let file = File::create_new(file_path)
        .into_diagnostic()
        .wrap_err("failed to create new file")?;

    let mut buffered_file = BufWriter::new(file);


    buffered_file
        .write_all(contents.as_bytes())
        .into_diagnostic()
        .wrap_err("failed to write contents to file")?;


    let mut file = buffered_file
        .into_inner()
        .into_diagnostic()
        .wrap_err("failed to flush buffered file writer")?;

    file.flush()
        .into_diagnostic()
        .wrap_err("failed to flush to file")?;

    Ok(())
}



struct ModRsGenerationInfo {
    needs_import_of_up_module: bool,
    needs_import_of_down_module: bool,
}

fn create_boilerplate_migration_and_rollback_scripts(
    migration_directory_path: &Path,
    migration_version: i64,
    migration_name: &str,
    migration_created_on: &DateTime<Utc>,
    up_script_type: GeneratedScriptType,
    rollback_script_type: Option<GeneratedScriptType>,
) -> Result<()> {
    let mut potential_mod_rs_generation_info: Option<ModRsGenerationInfo> = None;


    // Create up script, either SQL or Rust.
    let formatted_created_on = migration_created_on.to_rfc3339_opts(SecondsFormat::Secs, true);

    match up_script_type {
        GeneratedScriptType::Sql => {
            let boilerplate_contents = format!(
                "-- Migration {:04}: {}\n\
                 -- Created on: {}",
                migration_version, migration_name, formatted_created_on
            );

            write_str_to_new_file(
                migration_directory_path.join("up.sql").as_path(),
                &boilerplate_contents,
            )
            .wrap_err("failed to write to up.sql")?;
        }
        GeneratedScriptType::Rust => {
            let boilerplate_contents = format!(
                "//! Migration script for {:04}: {}\n\
                //! Created on: {}\n
                \n\
                use kolomoni_migrations_core::errors::MigrationApplyError;\n\
                use sqlx::PgConnection;\n\
                \n\
                #[kolomoni_migrations_macros::up]\n\
                pub async fn up(database_connection: &mut PgConnection) -> Result<(), MigrationApplyError> {{\n    \
                    todo!();\n\
                }}\n",
                migration_version,
                migration_name,
                formatted_created_on
            );

            write_str_to_new_file(
                migration_directory_path.join("up.rs").as_path(),
                &boilerplate_contents,
            )
            .wrap_err("failed to write to up.rs")?;

            potential_mod_rs_generation_info = Some(ModRsGenerationInfo {
                needs_import_of_up_module: true,
                needs_import_of_down_module: false,
            });
        }
    }


    // Create down script, either SQL or Rust.
    if let Some(rollback_script_type) = rollback_script_type {
        match rollback_script_type {
            GeneratedScriptType::Sql => {
                let boilerplate_contents = format!(
                    "-- Rollback script for migration {:04}: {}\n\
                     -- Created on: {}",
                    migration_version, migration_name, formatted_created_on
                );

                write_str_to_new_file(
                    migration_directory_path.join("down.sql").as_path(),
                    &boilerplate_contents,
                )
                .wrap_err("failed to write to down.sql")?;
            }
            GeneratedScriptType::Rust => {
                let boilerplate_contents = format!(
                    "//! Rollback script for migration {:04}: {}\n\
                    //! Created on: {}\n
                    \n\
                    use kolomoni_migrations_core::errors::MigrationRollbackError;\n\
                    use sqlx::PgConnection;\n\
                    \n\
                    #[kolomoni_migrations_macros::down]\n\
                    pub async fn down(database_connection: &mut PgConnection) -> Result<(), MigrationRollbackError> {{\n    \
                        todo!();\n\
                    }}\n",
                    migration_version,
                    migration_name,
                    formatted_created_on
                );

                write_str_to_new_file(
                    migration_directory_path.join("down.rs").as_path(),
                    &boilerplate_contents,
                )
                .wrap_err("failed to write to down.rs")?;

                match potential_mod_rs_generation_info {
                    Some(existing_info) => {
                        potential_mod_rs_generation_info = Some(ModRsGenerationInfo {
                            needs_import_of_down_module: true,
                            ..existing_info
                        });
                    }
                    None => {
                        potential_mod_rs_generation_info = Some(ModRsGenerationInfo {
                            needs_import_of_up_module: false,
                            needs_import_of_down_module: true,
                        });
                    }
                }
            }
        }
    }


    // If either of the scripts was a Rust script, we will have to create a mod.rs to reexport them as well.
    if let Some(mod_rs_generation_info) = potential_mod_rs_generation_info {
        let up_module_use_contents = if mod_rs_generation_info.needs_import_of_up_module {
            "pub(crate) mod up;\n"
        } else {
            ""
        };

        let down_module_use_contents = if mod_rs_generation_info.needs_import_of_down_module {
            "pub(crate) mod down;\n"
        } else {
            ""
        };

        let boilerplate_contents = format!(
            "//! Module file for migration {:04}: {}\n\
            //! Created on: {}\n
            \n\
            {}{}
            \n",
            migration_version,
            migration_name,
            formatted_created_on,
            up_module_use_contents,
            down_module_use_contents
        );


        write_str_to_new_file(
            migration_directory_path.join("mod.rs").as_path(),
            &boilerplate_contents,
        )
        .wrap_err("failed to write to mod.rs")?;
    }


    Ok(())
}


pub fn cli_generate(arguments: GenerateCommandArguments) -> Result<()> {
    let manager = crate::migrations::manager();


    let next_migration_version =
        if let Some(last_migration) = manager.migrations_without_status().last() {
            last_migration.identifier().version + 1
        } else {
            1
        };


    if let Some(migration_version_override) = arguments.migration_version {
        if migration_version_override < next_migration_version {
            // This means the user requested to generate a migration that would not, by its version,
            // not be at the end. This is invalid.
            return Err(miette!(
                "invalid --migration-version: must be larger than the most recent migration"
            ));
        }
    }


    let migration_version_to_generate = arguments
        .migration_version
        .unwrap_or(next_migration_version);

    let migration_directory_name_to_generate = MigrationIdentifier::new(
        migration_version_to_generate,
        arguments.migration_name.clone(),
    )
    .to_directory_name();


    let migration_directory_path = arguments
        .migrations_directory_path
        .join(&migration_directory_name_to_generate);

    if migration_directory_path.exists() {
        return Err(miette!(
            "Migration directory with the specified version and name already exists."
        ));
    }


    let migration_created_on = Utc::now();


    print!(
        "Creating migration \"{}\"...",
        migration_directory_name_to_generate
    );

    fs::create_dir_all(&migration_directory_path)
        .into_diagnostic()
        .wrap_err("failed to create migration directory")?;

    println!("  [Done!]");



    print!("Creating boilerplate scripts for the new migration...");

    create_boilerplate_migration_and_rollback_scripts(
        &migration_directory_path,
        migration_version_to_generate,
        &arguments.migration_name,
        &migration_created_on,
        arguments.up_script_type.unwrap_or(GeneratedScriptType::Sql),
        if !arguments.no_rollback {
            Some(
                arguments
                    .rollback_script_type
                    .unwrap_or(GeneratedScriptType::Sql),
            )
        } else {
            None
        },
    )?;

    println!("  [Done!]");


    // If enabled, create default migration.toml file.
    if !arguments.no_configuration_file {
        print!("Creating default migration.toml file for the new migration...");

        let default_configuration_file_contents = MigrationConfiguration::generate_template(
            migration_version_to_generate,
            &arguments.migration_name,
            &migration_created_on,
        );

        write_str_to_new_file(
            migration_directory_path
                .join(MigrationConfiguration::file_name_in_migration_directory())
                .as_path(),
            &default_configuration_file_contents,
        )
        .wrap_err("failed to write to migration.toml")?;

        println!("  [Done!]");
    }



    Ok(())
}
