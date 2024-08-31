use std::io::{self, Write};

use kolomoni_migrations_core::{connect_to_database, MigrationStatus};
use miette::{miette, Context, IntoDiagnostic, Result};

use crate::{cli::DownCommandArguments, commands::get_database_url_with_env_fallback};

pub fn cli_down(arguments: DownCommandArguments) -> Result<()> {
    let async_runtime = tokio::runtime::Runtime::new()
        .into_diagnostic()
        .wrap_err("failed to initialize tokio async runtime")?;

    async_runtime
        .block_on(cli_down_inner(arguments))
        .wrap_err("failed to run root async task to completion")
}


async fn cli_down_inner(arguments: DownCommandArguments) -> Result<()> {
    let manager = crate::migrations::manager();


    let database_url =
        get_database_url_with_env_fallback(arguments.database_url.as_ref(), "DATABASE_URL")?
            .ok_or_else(|| {
                miette!(
                    "either the --database-url argument or the DATABASE_URL \
                    environment variable must be specified"
                )
            })?;


    print!("Connecting to the PostgreSQL database...");

    let mut database_connection = connect_to_database(database_url.as_ref())
        .await
        .into_diagnostic()
        .wrap_err("failed to connect to database")?;

    println!("  [Connected!]");


    print!("Loading migrations...");

    manager
        .initialize_migration_tracking_in_database(&mut database_connection)
        .await
        .into_diagnostic()
        .wrap_err("failed to ensure migration tracking is set up")?;

    let migrations = manager
        .migrations_with_status(&mut database_connection)
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
        println!("No migrations to rollback: no migrations available.");

        return Ok(());
    }


    let version_to_rollback_to = arguments.rollback_to_version;

    // Verify the version exists.
    let selected_version_exists = migrations
        .iter()
        .any(|migration| migration.identifier().version == version_to_rollback_to);

    if !selected_version_exists && version_to_rollback_to != 0 {
        println!(
            "Unable to rollback to version {}: no such version exists.",
            version_to_rollback_to
        );

        return Ok(());
    }


    let mut migrations_to_rollback = Vec::new();

    for migration in migrations.iter().rev() {
        if !matches!(
            migration.status(),
            MigrationStatus::Applied { .. }
        ) {
            continue;
        }

        if migration.identifier().version > version_to_rollback_to {
            migrations_to_rollback.push(migration);
        }
    }

    if migrations_to_rollback.is_empty() {
        println!("No migrations to roll back: already at (or before) specified target version.");

        return Ok(());
    }


    println!(
        "Found {} migrations to rollback to reach version {}:",
        migrations_to_rollback.len(),
        version_to_rollback_to
    );
    for migration_to_rollback in &migrations_to_rollback {
        println!("  {}", migration_to_rollback.identifier());
    }
    println!();


    print!("Are you sure you want to continue? The migrations above will be rolled back. [y/N] ");
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

    for migration_to_roll_back in &migrations_to_rollback {
        print!(
            "Rolling back migration {}...",
            migration_to_roll_back.identifier()
        );

        migration_to_roll_back
            .execute_down(&mut database_connection)
            .await
            .into_diagnostic()?;

        println!("  [Done!]");
    }

    println!();
    println!(
        "All {} requested migrations have been rolled back, \
        database schema is now at version {}.",
        migrations_to_rollback.len(),
        version_to_rollback_to
    );


    Ok(())
}
