use std::borrow::Cow;

use thiserror::Error;

#[macro_use]
pub(crate) mod macros;

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

    #[error("database inconsistency: {}", .problem)]
    DatabaseInconsistencyError { problem: Cow<'static, str> },
}

impl QueryError {
    pub fn database_inconsistency<R>(problem: R) -> Self
    where
        R: Into<Cow<'static, str>>,
    {
        Self::DatabaseInconsistencyError {
            problem: problem.into(),
        }
    }
}



pub type QueryResult<R, E = QueryError> = Result<R, E>;


pub trait IntoStronglyTypedInternalModel {
    type InternalModel;

    fn into_strongly_typed_internal_model(self) -> Self::InternalModel;
}

pub trait TryIntoStronglyTypedInternalModel {
    type InternalModel;
    type Error;

    fn try_into_strongly_typed_internal_model(self) -> Result<Self::InternalModel, Self::Error>;
}



pub trait IntoExternalModel {
    type ExternalModel;

    fn into_external_model(self) -> Self::ExternalModel;
}

pub trait TryIntoExternalModel {
    type ExternalModel;
    type Error;

    fn try_into_external_model(self) -> Result<Self::ExternalModel, Self::Error>;
}


pub trait IntoInternalModel {
    type InternalModel;

    fn into_internal_model(self) -> Self::InternalModel;
}

pub trait TryIntoInternalModel {
    type InternalModel;
    type Error;

    fn try_into_internal_model(self) -> Result<Self::InternalModel, Self::Error>;
}
