mod model;
mod mutation;
mod query;

use std::borrow::Cow;

use kolomoni_auth::ArgonHasherError;
pub use model::*;
pub use mutation::*;
pub use query::*;
use thiserror::Error;

use crate::QueryError;



#[derive(Debug, Error)]
pub enum UserQueryError {
    #[error("sqlx error")]
    SqlxError {
        #[from]
        #[source]
        error: sqlx::Error,
    },

    #[error("model error: {}", .reason)]
    ModelError { reason: Cow<'static, str> },

    #[error("hasher error")]
    HasherError {
        #[from]
        #[source]
        error: ArgonHasherError,
    },

    #[error("database consistency error: {}", .reason)]
    DatabaseConsistencyError { reason: Cow<'static, str> },
}

impl From<QueryError> for UserQueryError {
    fn from(value: QueryError) -> Self {
        match value {
            QueryError::SqlxError { error } => Self::SqlxError { error },
            QueryError::ModelError { reason } => Self::ModelError { reason },
            QueryError::DatabaseInconsistencyError { problem: reason } => {
                Self::DatabaseConsistencyError { reason }
            }
        }
    }
}

pub type UserQueryResult<V> = Result<V, UserQueryError>;
