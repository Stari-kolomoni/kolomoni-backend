use std::io::{self, Write};

use miette::{miette, Context, IntoDiagnostic, Result};

use crate::{
    cli::UpCommandArguments,
    commands::get_database_url_with_env_fallback,
    connect_to_database,
    errors::RemoteMigrationError,
    models::{load_and_validate_migrations_with_status, MigrationStatus},
};


pub fn cli_up(arguments: UpCommandArguments) -> Result<()> {
    let async_runtime = tokio::runtime::Runtime::new()
        .into_diagnostic()
        .wrap_err("failed to initialize tokio async runtime")?;

    async_runtime
        .block_on(cli_up_inner(arguments))
        .wrap_err("failed to run root async task to completion")
}




pub async fn cli_up_inner(arguments: UpCommandArguments) -> Result<()> {
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
        .map_err(|error| RemoteMigrationError::UnableToAccessDatabase { error })
        .into_diagnostic()?;

    println!("  [Connected!]");


    print!("Loading local migrations...");

    let migrations = load_and_validate_migrations_with_status(
        &arguments.migrations_directory_path,
        &mut database_connection,
    )
    .await
    .into_diagnostic()
    .wrap_err("failed to load and validate migrations")?;

    println!(
        "  [Loaded {} migrations ({} already applied)]",
        migrations.len(),
        migrations
            .iter()
            .filter(|migration| matches!(migration.status, MigrationStatus::Applied { .. }))
            .count()
    );
    println!();

    if migrations.is_empty() {
        println!("No migrations to apply: no migrations available.");

        return Ok(());
    }



    let version_to_migrate_to = match arguments.migrate_to_version {
        Some(version_to_migrate_to) => version_to_migrate_to,
        None => {
            // PANIC SAFETY: We checked above that `migrations` is not empty.
            migrations.last().unwrap().identifer.version
        }
    };


    let mut migrations_to_apply = Vec::new();

    for migration in &migrations {
        if !matches!(migration.status, MigrationStatus::Pending) {
            continue;
        }

        if migration.identifer.version <= version_to_migrate_to {
            migrations_to_apply.push(migration);
        }
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
        println!("  {}", migration_to_apply.identifer);
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
        print!(
            "Applying migration {}...",
            migration_to_apply.identifer
        );

        migration_to_apply
            .apply(&mut database_connection)
            .await
            .into_diagnostic()?;

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
