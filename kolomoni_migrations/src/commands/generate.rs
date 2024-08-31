use std::{
    fs::{self, File},
    io::{BufWriter, Write},
};

use chrono::Utc;
use kolomoni_migrations_core::{
    configuration::MigrationConfiguration,
    identifier::MigrationIdentifier,
};
use miette::{miette, Context, IntoDiagnostic, Result};

use crate::cli::GenerateCommandArguments;



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


    let migration_directory_path_to_create = arguments
        .migrations_directory_path
        .join(&migration_directory_name_to_generate);

    if migration_directory_path_to_create.exists() {
        return Err(miette!(
            "Migration directory with the specified version and name already exists."
        ));
    }


    print!(
        "Creating migration \"{}\"...",
        migration_directory_name_to_generate
    );

    fs::create_dir_all(&migration_directory_path_to_create)
        .into_diagnostic()
        .wrap_err("failed to create migration directory")?;

    println!("  [Done!]");



    // TODO Needs ability to create templated up.rs / down.rs scripts as well.

    // Create empty up.sql.
    print!("Creating empty up.sql script for the new migration...");

    File::create_new(migration_directory_path_to_create.join("up.sql"))
        .into_diagnostic()
        .wrap_err("failed to create up.sql inside new migration directory")?;

    println!("  [Done!]");



    // If enabled, create empty down.sql.
    if !arguments.no_rollback {
        print!("Creating empty down.sql script for the new migration...");

        File::create_new(migration_directory_path_to_create.join("down.sql"))
            .into_diagnostic()
            .wrap_err("failed to create down.sql inside new migration directory")?;

        println!("  [Done!]");
    }

    // If enabled, create default migration.toml file.
    if !arguments.no_configuration_file {
        print!("Creating default migration.toml file for the new migration...");

        let migration_toml_file = File::create_new(
            migration_directory_path_to_create
                .join(MigrationConfiguration::file_name_in_migration_directory()),
        )
        .into_diagnostic()
        .wrap_err("failed to create migration.toml file inside the new migration directory")?;

        let mut buffered_file = BufWriter::new(migration_toml_file);


        buffered_file
            .write_all(
                MigrationConfiguration::generate_template(
                    migration_version_to_generate,
                    &arguments.migration_name,
                    Utc::now(),
                )
                .as_bytes(),
            )
            .into_diagnostic()
            .wrap_err("failed to write default migration.toml contents to file")?;


        let mut migration_toml_file = buffered_file
            .into_inner()
            .into_diagnostic()
            .wrap_err("failed to flush buffered file writer for migration.toml")?;

        migration_toml_file
            .flush()
            .into_diagnostic()
            .wrap_err("failed to flush to migration.toml file")?;

        println!("  [Done!]");
    }



    Ok(())
}
