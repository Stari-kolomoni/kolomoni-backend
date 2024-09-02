use std::{borrow::Cow, error::Error};

use thiserror::Error;

use crate::{identifier::MigrationIdentifier, sha256::Sha256Hash};



#[derive(Debug, Error)]
pub enum RemoteMigrationError {
    #[error("failed to execute query in database")]
    QueryFailed {
        #[source]
        error: sqlx::Error,
    },

    #[error(
        "invalid row {} encountered in migration tracking table: {}",
        .identifier,
        .reason
    )]
    InvalidRow {
        identifier: MigrationIdentifier,

        reason: Cow<'static, str>,
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

    #[error("other error")]
    OtherError {
        #[source]
        error: Box<dyn Error + Send + Sync + 'static>,
    },
}



#[derive(Debug, Error)]
pub enum MigrationRollbackError {
    #[error("no rollback script")]
    RollbackUndefined,

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
pub enum InitializeMigrationDirectoryError {
    #[error("provided migrations directory path exists, but is not a directory")]
    NotADirectory,

    #[error("unable to create missing migrations directory")]
    UnableToCreate {
        #[source]
        error: std::io::Error,
    },
}

#[derive(Debug, Error)]
pub enum InitializeMigrationTrackingError {
    #[error("failed to create migration tracking table in database")]
    UnableToCreateTable {
        #[source]
        error: sqlx::Error,
    },
}

#[derive(Debug, Error)]
pub enum StatusError {
    #[error("failed to load migration from database")]
    RemoteMigrationError(
        #[from]
        #[source]
        RemoteMigrationError,
    ),

    #[error(
        "migration exists in the database, but its local \
        (embedded) counterpart cannot be found: {}",
        .identifier
    )]
    MigrationDoesNotExistLocally { identifier: MigrationIdentifier },

    #[error(
        "embedded and remote migration don't match due to different hashes (version {}): \
        {} vs {} (up), {:?} vs {:?} (down)",
        .identifier,
        .remote_up_script_sha256_hash,
        .embedded_up_script_sha256_hash,
        .remote_down_script_sha256_hash,
        .embedded_down_script_sha256_hash
    )]
    HashMismatch {
        identifier: MigrationIdentifier,

        remote_up_script_sha256_hash: Sha256Hash,

        embedded_up_script_sha256_hash: Sha256Hash,

        remote_down_script_sha256_hash: Option<Sha256Hash>,

        embedded_down_script_sha256_hash: Option<Sha256Hash>,
    },

    #[error(
        "embedded and remote migration don't match due to different names \
        being used for the same version: {} and {} are used for version {}",
        .embedded_migration_name,
        .remote_migration_name,
        .version
    )]
    NameMismatch {
        version: i64,

        remote_migration_name: String,

        embedded_migration_name: String,
    },
}
