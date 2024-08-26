use std::{
    env::{self, VarError},
    fs,
    path::Path,
};

use miette::{miette, Context, IntoDiagnostic, Result};
use sqlx::{Connection, PgConnection};

use crate::{cli::InitializeCommandArguments, connect_to_database, errors::RemoteMigrationError};




pub(crate) async fn set_up_migration_table_if_needed(
    connection: &mut PgConnection,
) -> Result<(), RemoteMigrationError> {
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS kolomoni.schema_migrations (
            version bigint NOT NULL,
            name text NOT NULL,
            up_sql_sha256_hash bytea NOT NULL,
            down_sql_sha256_hash bytea,
            applied_at timestamp with time zone NOT NULL,
            CONSTRAINT pk__schema_migrations PRIMARY KEY (version)
        )
        "#
    )
    .execute(&mut *connection)
    .await
    .map_err(|error| RemoteMigrationError::UnableToCreateMigrationsTable { error })?;

    Ok(())
}



pub fn cli_initialize(arguments: InitializeCommandArguments) -> Result<()> {
    let async_runtime = tokio::runtime::Runtime::new()
        .into_diagnostic()
        .wrap_err("failed to initialize tokio async runtime")?;

    async_runtime
        .block_on(cli_initialize_inner(arguments))
        .wrap_err("failed to run root async task to completion")
}

async fn cli_initialize_inner(arguments: InitializeCommandArguments) -> Result<()> {
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
        .map_err(|error| RemoteMigrationError::UnableToAccessDatabase { error })
        .into_diagnostic()?;

    println!("  [Connected!]");


    print!("Setting up migration table if missing...");
    set_up_migration_table_if_needed(&mut database_connection)
        .await
        .into_diagnostic()
        .wrap_err("failed to set up migrations table in database")?;
    println!("  [Done!]");



    print!("Disconnecting from the database...");

    database_connection
        .close()
        .await
        .into_diagnostic()
        .wrap_err("failed to close the database connection")?;

    println!("  [Done!]");



    print!("Checking for migrations directory, creating one if needed...");

    create_migrations_directory_if_missing(&arguments.migrations_directory_path)?;

    println!("  [Done!]");


    Ok(())
}


fn create_migrations_directory_if_missing(migrations_directory_path: &Path) -> Result<()> {
    if migrations_directory_path.exists() {
        if !migrations_directory_path.is_dir() {
            return Err(miette!(
                "the provided migrations path exists, but is not a directory"
            ));
        }

        return Ok(());
    }

    fs::create_dir_all(migrations_directory_path)
        .into_diagnostic()
        .wrap_err("failed to create migrations directory")?;

    Ok(())
}
