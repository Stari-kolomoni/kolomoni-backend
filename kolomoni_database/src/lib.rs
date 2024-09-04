use std::borrow::Cow;

use thiserror::Error;

pub mod entities;


#[derive(Debug, Error)]
pub enum QueryError {
    #[error("sqlx error")]
    SqlxError {
        #[from]
        #[source]
        error: sqlx::Error,
    },

    #[error("model error: {}", .reason)]
    ModelError { reason: Cow<'static, str> },
}

pub type QueryResult<R, E = QueryError> = Result<R, E>;


pub(crate) trait IntoModel {
    type Model;

    fn into_model(self) -> Self::Model;
}

pub(crate) trait TryIntoModel {
    type Model;
    type Error;

    fn try_into_model(self) -> Result<Self::Model, Self::Error>;
}
