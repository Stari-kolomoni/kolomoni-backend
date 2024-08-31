use std::{borrow::Cow, error::Error};

use thiserror::Error;

use crate::identifier::MigrationIdentifier;



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


/*
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
 */
