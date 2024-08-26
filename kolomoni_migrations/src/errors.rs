use std::{borrow::Cow, path::PathBuf};

use thiserror::Error;

use crate::{
    models::{
        local::configuration::MigrationConfigurationError,
        InvalidMigrationIdentifierError,
        MigrationIdentifier,
    },
    sha256::Sha256Hash,
};



#[derive(Debug, Error)]
pub enum RemoteMigrationError {
    #[error("unable to access database")]
    UnableToAccessDatabase {
        #[source]
        error: sqlx::Error,
    },

    #[error("failed to create \"schema_migrations\" table in database")]
    UnableToCreateMigrationsTable {
        #[source]
        error: sqlx::Error,
    },

    #[error(
        "invalid row {} encountered in migration table: {}",
        .identifier,
        .reason
    )]
    InvalidRow {
        identifier: MigrationIdentifier,

        reason: Cow<'static, str>,
    },
}



#[derive(Debug, Error)]
pub enum LocalMigrationError {
    #[error(
        "invalid structure for migration entry at \"{}\": {}",
        .migration_directory_path.display(),
        .reason
    )]
    InvalidMigrationStructure {
        migration_directory_path: PathBuf,

        reason: Cow<'static, str>,
    },

    #[error(
        "invalid local migration identifier \"{}\"",
        .identifier,
    )]
    InvalidMigrationIdentifier {
        identifier: String,

        #[source]
        error: InvalidMigrationIdentifierError,
    },

    #[error(
        "migration version {} is not unique",
        .version
    )]
    MigrationVersionIsNotUnique { version: i64 },

    #[error(
        "failed to read migrations directory at \"{}\"",
        .directory_path.display()
    )]
    UnableToScanMigrationsDirectory {
        directory_path: PathBuf,

        #[source]
        error: fs_more::error::DirectoryScanError,
    },

    #[error(
        "failed to read migration at \"{}\"",
        .path.display()
    )]
    UnableToReadMigration {
        path: PathBuf,

        #[source]
        error: std::io::Error,
    },

    #[error(
        "failed to parse migration configuration for {}",
        .identifier
    )]
    ConfigurationError {
        identifier: MigrationIdentifier,

        #[source]
        error: MigrationConfigurationError,
    },
}



#[derive(Debug, Error)]
pub enum MigrationApplyError {
    #[error("failed while executing query")]
    FailedToExecuteQuery {
        #[source]
        error: sqlx::Error,
    },

    #[error("failed while setting up or commiting transaction")]
    FailedToPerformTransaction {
        #[source]
        error: sqlx::Error,
    },
}



#[derive(Debug, Error)]
pub enum MigrationRollbackError {
    #[error("migration cannot be rolled back")]
    MigrationCannotBeRolledBack,

    #[error("failed while executing query")]
    FailedToExecuteQuery {
        #[source]
        error: sqlx::Error,
    },

    #[error("failed while setting up or commiting transaction")]
    FailedToPerformTransaction {
        #[source]
        error: sqlx::Error,
    },
}



#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("database error encountered")]
    RemoteError {
        #[from]
        #[source]
        error: RemoteMigrationError,
    },

    #[error("local migration error encountered")]
    LocalError {
        #[from]
        #[source]
        error: LocalMigrationError,
    },

    #[error(
        "migration versions must be unique, but found at least two with version {}",
        .version
    )]
    MigrationVersionIsNotUnique { version: i64 },

    #[error(
        "remote and local migration {} don't match due to different hashes: {} vs {} (up), {:?} vs {:?} (down)",
        .identifier,
        .remote_up_sql_sha256_hash,
        .local_up_sql_sha256_hash,
        .remote_down_sql_sha256_hash,
        .local_down_sql_sha256_hash
    )]
    RemoteAndLocalMigrationHasDifferentHash {
        identifier: MigrationIdentifier,

        remote_up_sql_sha256_hash: Sha256Hash,

        local_up_sql_sha256_hash: Sha256Hash,

        remote_down_sql_sha256_hash: Option<Sha256Hash>,

        local_down_sql_sha256_hash: Option<Sha256Hash>,
    },

    #[error(
        "migration exists in the database, but its local counterpart is no longer available: {}",
        .identifier
    )]
    MigrationNoLongerExistsLocally { identifier: MigrationIdentifier },

    #[error(
        "remote migration {} is invalid: {}",
        .identifier,
        .reason
    )]
    RemoteMigrationIsInvalid {
        identifier: MigrationIdentifier,

        reason: Cow<'static, str>,
    },
}
