use kolomoni_migrations_core::MigrationStatus;
use miette::{miette, Context, IntoDiagnostic, Result};

use crate::cli::StatusCommandArguments;

pub fn cli_status(arguments: StatusCommandArguments) -> Result<()> {
    let async_runtime = tokio::runtime::Runtime::new()
        .into_diagnostic()
        .wrap_err("failed to initialize tokio async runtime")?;

    async_runtime
        .block_on(cli_status_inner(arguments))
        .wrap_err("failed to run root async task to completion")
}

async fn cli_status_inner(arguments: StatusCommandArguments) -> Result<()> {
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


    let Some(connection_to_load_migrations_with) = privileged_user_db_connection_options
        .as_ref()
        .or(normal_user_db_connection_options.as_ref())
    else {
        return Err(miette!("Invalid arguments: need at least one of the available database connection options (normal or privileged)."));
    };


    print!("Loading migrations...");

    let migration_collection = manager
        .migrations_with_status_with_fallback(connection_to_load_migrations_with)
        .await
        .into_diagnostic()
        .wrap_err("failed to load migrations")?;


    for migration in migration_collection.migrations() {
        println!();
        println!(
            "M{:03}: {}",
            migration.identifier().version,
            migration.identifier().name
        );
        println!(
            "      Status: {}",
            match migration.status() {
                MigrationStatus::Pending => "not applied".to_string(),
                MigrationStatus::Applied { at, integrity } => {
                    let formatted_time = at.format("%Y-%m-%dT%H:%M:%S");
                    let formatted_integrity = if integrity.has_full_integrity() {
                        "Integrity OK.".to_string()
                    } else {
                        format!(
                            "Integrity NOT OK: up script is {}, down script is {}",
                            if !integrity.up_hash_matches {
                                "NOT OK"
                            } else {
                                "OK"
                            },
                            if !integrity.down_hash_matches {
                                "NOT OK"
                            } else {
                                "OK"
                            }
                        )
                    };

                    format!(
                        "applied at {}\n      Integrity: {}",
                        formatted_time, formatted_integrity
                    )
                }
            }
        )
    }


    Ok(())
}
