use std::io::{self, Write};

use kolomoni_migrations_core::{
    migrations::MigrationsWithStatusOptions,
    DatabaseConnectionManager,
    MigrationStatus,
};
use miette::{miette, Context, IntoDiagnostic, Result};

use crate::cli::UpCommandArguments;


pub fn cli_up(arguments: UpCommandArguments) -> Result<()> {
    let async_runtime = tokio::runtime::Runtime::new()
        .into_diagnostic()
        .wrap_err("failed to initialize tokio async runtime")?;

    async_runtime
        .block_on(cli_up_inner(arguments))
        .wrap_err("failed to run root async task to completion")
}




pub async fn cli_up_inner(arguments: UpCommandArguments) -> Result<()> {
    let manager = crate::migrations::manager();


    let normal_user_db_connection_options = arguments
        .database
        .database_connection_options_for_normal_user()
        .into_diagnostic()
        .wrap_err("failed to obtain normal database connection info")?;

    let privileged_user_db_connection_options = arguments
        .database
        .database_connection_options_for_privileged_user()
        .into_diagnostic()
        .wrap_err("failed to obtain privileged database connection info")?;


    let mut connection_manager = DatabaseConnectionManager::new();


    /*
    print!("Connecting to the PostgreSQL database...");

    let mut database_connection = normal_user_db_connection_options
        .connect()
        .await
        .into_diagnostic()
        .wrap_err("failed to connect to database")?;

    println!("  [Connected!]"); */


    print!("Loading migrations...");

    let migrations = manager
        .migrations_with_status_with_fallback(
            normal_user_db_connection_options.as_ref(),
            MigrationsWithStatusOptions {
                require_up_hashes_match: true,
                require_down_hashes_match: true,
            },
        )
        .await
        .into_diagnostic()
        .wrap_err("failed to load migrations")?;

    println!(
        "  [Loaded {} migrations ({} already applied)]",
        migrations.len(),
        migrations
            .iter()
            .filter(|migration| matches!(
                migration.status(),
                MigrationStatus::Applied { .. }
            ))
            .count()
    );
    println!();

    if migrations.is_empty() {
        println!("No migrations to apply: no migrations available.");

        return Ok(());
    }



    let version_to_migrate_to = match arguments.migrate_to_version {
        Some(version_to_migrate_to) => {
            // Verify the version exists.
            let selected_version_exists = migrations
                .iter()
                .any(|migration| migration.identifier().version == version_to_migrate_to);

            if !selected_version_exists {
                println!(
                    "Unable to migrate to version {}: no such version exists.",
                    version_to_migrate_to
                );

                return Ok(());
            }

            version_to_migrate_to
        }
        None => {
            // PANIC SAFETY: We checked above that `migrations` is not empty.
            migrations.last().unwrap().identifier().version
        }
    };


    let mut migrations_to_apply = Vec::new();

    for migration in &migrations {
        if !matches!(migration.status(), MigrationStatus::Pending) {
            continue;
        }

        if migration.identifier().version > version_to_migrate_to {
            continue;
        }


        if migration.configuration().run_as_privileged_user {
            if privileged_user_db_connection_options.is_none() {
                println!(
                    "Unable to apply planned migration set: \
                    migration {} requires privileged access, but --privileged-user-and-password \
                    or the equivalent environment variable has not been set.",
                    migration.identifier()
                );

                return Ok(());
            }
        } else if normal_user_db_connection_options.is_none() {
            println!(
                "Unable to apply planned migration set: \
                migration {} requires normal (non-privileged) access, but --normal-user-and-password \
                or the equivalent environment variable has not been set.",
                migration.identifier()
            );

            return Ok(());
        }

        migrations_to_apply.push(migration);
    }

    if migrations_to_apply.is_empty() {
        println!("No migrations to apply: already at (or past) specified target version.");

        return Ok(());
    }


    println!(
        "Found {} migrations to apply to reach version {}:",
        migrations_to_apply.len(),
        version_to_migrate_to
    );
    for migration_to_apply in &migrations_to_apply {
        println!("  {}", migration_to_apply.identifier());
    }
    println!();

    print!("Are you sure you want to continue? The migrations above will be applied. [y/N] ");
    io::stdout()
        .flush()
        .into_diagnostic()
        .wrap_err("failed to flush terminal output")?;

    let mut user_response = String::new();
    io::stdin()
        .read_line(&mut user_response)
        .into_diagnostic()
        .wrap_err("failed to read user terminal input")?;

    if user_response.trim_end().to_ascii_lowercase() != "y" {
        return Err(miette!("User aborted command."));
    }


    println!();

    for migration_to_apply in &migrations_to_apply {
        let database_connection = match migration_to_apply.configuration().run_as_privileged_user {
            true => {
                match connection_manager.get_existing_privileged_user_connection() {
                    Some(connection) => connection,
                    None => {
                        print!("Establishing database connection (as privileged user)...");

                        let connection = connection_manager.establish_privileged_user_connection(
                            privileged_user_db_connection_options.as_ref()
                            // PANIC SAFETY: A migration planner loop above must check whether the user has specified the connection options.
                            .expect("expected the function to check whether the user has specified privileged user beforehand")
                        ).await
                            .into_diagnostic()
                            .wrap_err("failed to establish database connection for privileged user")?;

                        println!("  [Done!]");

                        connection
                    }
                }
            }
            false => {
                match connection_manager.get_existing_normal_user_connection() {
                    Some(connection) => connection,
                    None => {
                        print!("Establishing database connection (as normal user)...");

                        let connection = connection_manager.establish_normal_user_connection(
                            normal_user_db_connection_options.as_ref()
                            // PANIC SAFETY: A migration planner loop above must check whether the user has specified the connection options.
                            .expect("expected the function to check whether the user has specified the normal user beforehand")
                        ).await
                            .into_diagnostic()
                            .wrap_err("failed to establish database connection for normal user")?;

                        println!("  [Done!]");

                        connection
                    }
                }
            }
        };


        print!(
            "Applying migration {}...",
            migration_to_apply.identifier()
        );

        migration_to_apply
            .execute_up(database_connection)
            .await
            .into_diagnostic()
            .wrap_err_with(|| {
                miette!(
                    "failed to execute migration {}",
                    migration_to_apply.identifier()
                )
            })?;

        println!("  [Done!]");
    }

    println!();
    println!(
        "All {} requested migrations applied, database schema is now at version {}.",
        migrations_to_apply.len(),
        version_to_migrate_to
    );

    Ok(())
}
