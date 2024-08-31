use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use errors::{MigrationApplyError, MigrationRollbackError, RemoteMigrationError};
use identifier::MigrationIdentifier;
use migrations::BoxedMigrationFn;
use remote::RemoteMigrationType;
use sha256::Sha256Hash;
use sqlx::{Connection, Executor, PgConnection};

pub mod configuration;
pub mod errors;
pub mod identifier;
pub mod migrations;
pub(crate) mod remote;
pub mod sha256;


/// Descibes a migration's status: either pending or applied (at some moment in time).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MigrationStatus {
    /// The migration is pending, which means its details reside in the migrations directory
    /// on disk, but it hasn't yet been applied to the database.
    Pending,

    /// The migration has already been applied to the database
    Applied {
        /// When the migration had been applied.
        at: DateTime<Utc>,
    },
}

pub async fn connect_to_database(database_url: &str) -> Result<PgConnection, sqlx::Error> {
    sqlx::PgConnection::connect(database_url).await
}

pub struct UpScriptDetails<'h> {
    r#type: RemoteMigrationType,
    sha256_hash: &'h Sha256Hash,
}

pub struct DownScriptDetails<'h> {
    r#type: RemoteMigrationType,
    sha256_hash: &'h Sha256Hash,
}

async fn insert_migration_tracking_row(
    database_connection: &mut PgConnection,
    migration_identifier: &MigrationIdentifier,
    up_script: UpScriptDetails<'_>,
    down_script: Option<DownScriptDetails<'_>>,
    applied_at: DateTime<Utc>,
    execution_time: Option<Duration>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO kolomoni.schema_migrations \
        (version, name, up_script_type, up_script_sha256_hash, \
        down_script_type, down_script_sha256_hash, \
        applied_at, execution_time_milliseconds)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(migration_identifier.version)
    .bind(migration_identifier.name.as_str())
    .bind(up_script.r#type.as_str())
    .bind(up_script.sha256_hash.as_slice())
    .bind(down_script.as_ref().map(|script| script.r#type.as_str()))
    .bind(
        down_script
            .as_ref()
            .map(|script| script.sha256_hash.as_slice()),
    )
    .bind(applied_at)
    .bind(
        execution_time
            .map(|time| time.as_millis() as i64)
            .unwrap_or(-1),
    )
    .execute(database_connection)
    .await?;

    Ok(())
}

async fn update_execution_time_in_migration_tracking_row(
    database_connection: &mut PgConnection,
    migration_version: i64,
    execution_time: Duration,
) -> Result<(), MigrationApplyError> {
    sqlx::query(
        "UPDATE kolomoni.schema_migrations \
        SET execution_time_milliseconds = $1 \
        WHERE version = $2",
    )
    .bind(execution_time.as_millis() as i64)
    .bind(migration_version)
    .execute(&mut *database_connection)
    .await
    .map_err(|error| MigrationApplyError::FailedToExecuteQuery { error })?;

    Ok(())
}

async fn remove_migration_tracking_row(
    database_connection: &mut PgConnection,
    migration_version: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM kolomoni.schema_migrations WHERE version = $1")
        .bind(migration_version)
        .execute(database_connection)
        .await?;

    Ok(())
}


async fn execute_up_sql_and_update_migrations_table(
    database_connection: &mut PgConnection,
    migration_identifier: &MigrationIdentifier,
    up_sql: &str,
    up_sql_sha256_hash: &Sha256Hash,
    down: Option<DownScriptDetails<'_>>,
) -> Result<(), MigrationApplyError> {
    let applied_at = Utc::now();

    // Executes the entire up.sql script of the migration.
    // FIXME a concurrent index can't be created for some reason, even when there is no transaction
    database_connection
        .execute(up_sql)
        .await
        .map_err(|error| MigrationApplyError::FailedToExecuteQuery { error })?;


    // Insert a a new row in the schema_migrations table that corresponds
    // to the migration that was just applied.
    insert_migration_tracking_row(
        database_connection,
        migration_identifier,
        UpScriptDetails {
            r#type: RemoteMigrationType::Sql,
            sha256_hash: up_sql_sha256_hash,
        },
        down,
        applied_at,
        None,
    )
    .await
    .map_err(|error| MigrationApplyError::FailedToExecuteQuery { error })?;

    Ok(())
}


pub(crate) async fn apply_sql_migration(
    database_connection: &mut PgConnection,
    run_in_transaction: bool,
    migration_identifier: &MigrationIdentifier,
    up_sql: &str,
    up_sql_sha256_hash: &Sha256Hash,
    down: Option<DownScriptDetails<'_>>,
) -> Result<(), MigrationApplyError> {
    let started_at = Instant::now();

    if run_in_transaction {
        let mut transaction = database_connection
            .begin()
            .await
            .map_err(|error| MigrationApplyError::FailedToPerformTransaction { error })?;

        execute_up_sql_and_update_migrations_table(
            &mut transaction,
            migration_identifier,
            up_sql,
            up_sql_sha256_hash,
            down,
        )
        .await?;

        transaction
            .commit()
            .await
            .map_err(|error| MigrationApplyError::FailedToPerformTransaction { error })?;
    } else {
        execute_up_sql_and_update_migrations_table(
            database_connection,
            migration_identifier,
            up_sql,
            up_sql_sha256_hash,
            down,
        )
        .await?;
    }


    let execution_time = started_at.elapsed();

    update_execution_time_in_migration_tracking_row(
        database_connection,
        migration_identifier.version,
        execution_time,
    )
    .await?;


    Ok(())
}



async fn execute_up_script_and_update_migrations_table<'c>(
    database_connection: &mut PgConnection,
    migration_identifier: &MigrationIdentifier,
    up_fn: &BoxedMigrationFn<'c, MigrationApplyError>,
    up_fn_sha256_hash: &Sha256Hash,
    down: Option<DownScriptDetails<'_>>,
) -> Result<(), MigrationApplyError> {
    let applied_at = Utc::now();

    up_fn(database_connection).await?;

    // Insert a a new row in the schema_migrations table that corresponds
    // to the migration that was just applied.
    insert_migration_tracking_row(
        database_connection,
        migration_identifier,
        UpScriptDetails {
            r#type: RemoteMigrationType::Rust,
            sha256_hash: up_fn_sha256_hash,
        },
        down,
        applied_at,
        None,
    )
    .await
    .map_err(|error| MigrationApplyError::FailedToExecuteQuery { error })?;

    Ok(())
}


pub(crate) async fn apply_rust_migration<'c>(
    database_connection: &mut PgConnection,
    run_in_transaction: bool,
    migration_identifier: &MigrationIdentifier,
    up_fn: &BoxedMigrationFn<'c, MigrationApplyError>,
    up_fn_sha256_hash: &Sha256Hash,
    down: Option<DownScriptDetails<'_>>,
) -> Result<(), MigrationApplyError> {
    let started_at = Instant::now();

    if run_in_transaction {
        let mut transaction = database_connection
            .begin()
            .await
            .map_err(|error| MigrationApplyError::FailedToPerformTransaction { error })?;

        execute_up_script_and_update_migrations_table(
            &mut transaction,
            migration_identifier,
            up_fn,
            up_fn_sha256_hash,
            down,
        )
        .await?;

        transaction
            .commit()
            .await
            .map_err(|error| MigrationApplyError::FailedToPerformTransaction { error })?;
    } else {
        execute_up_script_and_update_migrations_table(
            database_connection,
            migration_identifier,
            up_fn,
            up_fn_sha256_hash,
            down,
        )
        .await?;
    }


    let execution_time = started_at.elapsed();

    update_execution_time_in_migration_tracking_row(
        database_connection,
        migration_identifier.version,
        execution_time,
    )
    .await?;


    Ok(())
}


async fn execute_down_sql_and_update_migrations_table(
    database_connection: &mut PgConnection,
    migration_identifier: &MigrationIdentifier,
    down_sql: &str,
) -> Result<(), MigrationRollbackError> {
    // Executes the entire down.sql script of the migration.
    database_connection
        .execute(down_sql)
        .await
        .map_err(|error| MigrationRollbackError::FailedToExecuteQuery { error })?;


    // Deletes the corresponding row in the schema_migrations table that tracked
    // the migration being applied.
    remove_migration_tracking_row(database_connection, migration_identifier.version)
        .await
        .map_err(|error| MigrationRollbackError::FailedToExecuteQuery { error })?;

    Ok(())
}


pub(crate) async fn rollback_sql_migration(
    database_connection: &mut PgConnection,
    run_in_transaction: bool,
    migration_identifier: &MigrationIdentifier,
    down_sql: &str,
) -> Result<(), MigrationRollbackError> {
    if run_in_transaction {
        let mut transaction = database_connection
            .begin()
            .await
            .map_err(|error| MigrationRollbackError::FailedToPerformTransaction { error })?;

        execute_down_sql_and_update_migrations_table(
            &mut transaction,
            migration_identifier,
            down_sql,
        )
        .await?;

        transaction
            .commit()
            .await
            .map_err(|error| MigrationRollbackError::FailedToPerformTransaction { error })?;
    } else {
        execute_down_sql_and_update_migrations_table(
            database_connection,
            migration_identifier,
            down_sql,
        )
        .await?;
    }

    Ok(())
}


async fn execute_down_script_and_update_migrations_table<'c>(
    database_connection: &'c mut PgConnection,
    migration_identifier: &MigrationIdentifier,
    down_fn: &BoxedMigrationFn<'c, MigrationRollbackError>,
) -> Result<(), MigrationRollbackError> {
    down_fn(database_connection).await?;

    // Deletes the corresponding row in the schema_migrations table that tracked
    // the migration being applied.
    remove_migration_tracking_row(database_connection, migration_identifier.version)
        .await
        .map_err(|error| MigrationRollbackError::FailedToExecuteQuery { error })?;

    Ok(())
}

pub(crate) async fn rollback_rust_migration<'c>(
    database_connection: &mut PgConnection,
    run_in_transaction: bool,
    migration_identifier: &MigrationIdentifier,
    down_fn: &BoxedMigrationFn<'c, MigrationRollbackError>,
) -> Result<(), MigrationRollbackError> {
    if run_in_transaction {
        let mut transaction = database_connection
            .begin()
            .await
            .map_err(|error| MigrationRollbackError::FailedToPerformTransaction { error })?;

        execute_down_script_and_update_migrations_table(
            &mut transaction,
            migration_identifier,
            down_fn,
        )
        .await?;

        transaction
            .commit()
            .await
            .map_err(|error| MigrationRollbackError::FailedToPerformTransaction { error })?;
    } else {
        execute_down_script_and_update_migrations_table(
            database_connection,
            migration_identifier,
            down_fn,
        )
        .await?;
    }

    Ok(())
}


pub async fn set_up_migration_table_if_needed(
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
    .map_err(|error| RemoteMigrationError::QueryFailed { error })?;

    Ok(())
}
