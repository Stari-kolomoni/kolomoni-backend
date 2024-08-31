use std::env::{self, VarError};

use kolomoni_migrations_core::connect_to_database;
use miette::{miette, Context, IntoDiagnostic, Result};
use sqlx::Connection;

use crate::cli::InitializeCommandArguments;




pub fn cli_initialize(arguments: InitializeCommandArguments) -> Result<()> {
    let async_runtime = tokio::runtime::Runtime::new()
        .into_diagnostic()
        .wrap_err("failed to initialize tokio async runtime")?;

    async_runtime
        .block_on(cli_initialize_inner(arguments))
        .wrap_err("failed to run root async task to completion")
}

async fn cli_initialize_inner(arguments: InitializeCommandArguments) -> Result<()> {
    let manager = crate::migrations::manager();


    let database_url = match arguments.database_url {
        Some(database_url_from_cli) => database_url_from_cli,
        None => match env::var("DATABASE_URL") {
            Ok(database_url_from_env) => database_url_from_env,
            Err(error) => match error {
                VarError::NotPresent => {
                    return Err(miette!(
                        "either the --database-url argument or the DATABASE_URL \
                        environment variable must be specified"
                    ));
                }
                VarError::NotUnicode(_) => {
                    return Err(miette!(
                        "the DATABASE_URL environment variable is not valid Unicode"
                    ));
                }
            },
        },
    };

    print!("Connecting to the PostgreSQL database...");

    let mut database_connection = connect_to_database(database_url.as_str())
        .await
        .into_diagnostic()
        .wrap_err("unable to conneect to database")?;

    println!("  [Connected!]");


    print!("Setting up migration table if missing...");

    manager
        .initialize_migration_tracking_in_database(&mut database_connection)
        .await
        .into_diagnostic()
        .wrap_err("failed to initialize migration table in database")?;

    println!("  [Done!]");



    print!("Disconnecting from the database...");

    database_connection
        .close()
        .await
        .into_diagnostic()
        .wrap_err("failed to close the database connection")?;

    println!("  [Done!]");



    print!("Checking for migrations directory, creating one if needed...");

    manager
        .initialize_migrations_directory(&arguments.migrations_directory_path)
        .into_diagnostic()
        .wrap_err("failed to initialize migrations directory on disk")?;

    println!("  [Done!]");


    Ok(())
}
