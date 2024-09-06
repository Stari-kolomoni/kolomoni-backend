use std::borrow::Cow;

use futures_core::stream::BoxStream;
use kolomoni_auth::{ArgonHasher, ArgonHasherError};
use kolomoni_core::id::UserId;
use sqlx::PgConnection;
use thiserror::Error;

use crate::{IntoModel, QueryError, QueryResult};


#[derive(Debug, Error)]
pub enum UserCredentialValidationError {
    #[error("sqlx error")]
    SqlxError {
        #[source]
        error: sqlx::Error,
    },

    #[error("model error: {}", .reason)]
    ModelError { reason: Cow<'static, str> },

    #[error("hasher error")]
    HasherError {
        #[source]
        error: ArgonHasherError,
    },
}

impl From<QueryError> for UserCredentialValidationError {
    fn from(value: QueryError) -> Self {
        match value {
            QueryError::SqlxError { error } => Self::SqlxError { error },
            QueryError::ModelError { reason } => Self::ModelError { reason },
        }
    }
}



type RawUserStream<'c> = BoxStream<'c, Result<super::IntermediateModel, sqlx::Error>>;

create_async_stream_wrapper!(
    pub struct UserStream<'c>;
    transforms stream RawUserStream<'c> => stream of QueryResult<super::Model>:
        |value|
            value.map(|result| {
                result
                    .map(super::IntermediateModel::into_model)
                    .map_err(|error| QueryError::SqlxError { error })
            })
);




pub struct Query;

impl Query {
    pub async fn get_user_by_id(
        connection: &mut PgConnection,
        user_id: UserId,
    ) -> QueryResult<Option<super::Model>> {
        let optional_intermediate_model = sqlx::query_as!(
            super::IntermediateModel,
            "SELECT \
                id, username, display_name, hashed_password, \
                joined_at, last_modified_at, last_active_at \
            FROM kolomoni.user \
            WHERE id = $1",
            user_id.into_inner()
        )
        .fetch_optional(connection)
        .await?;

        Ok(optional_intermediate_model.map(super::IntermediateModel::into_model))
    }

    pub async fn get_user_by_username<U>(
        connection: &mut PgConnection,
        username: U,
    ) -> QueryResult<Option<super::Model>>
    where
        U: AsRef<str>,
    {
        let optional_intermediate_model = sqlx::query_as!(
            super::IntermediateModel,
            "SELECT \
                id, username, display_name, hashed_password, \
                joined_at, last_modified_at, last_active_at \
            FROM kolomoni.user \
            WHERE username = $1",
            username.as_ref()
        )
        .fetch_optional(connection)
        .await?;

        Ok(optional_intermediate_model.map(super::IntermediateModel::into_model))
    }

    pub async fn exists_by_id(connection: &mut PgConnection, user_id: UserId) -> QueryResult<bool> {
        sqlx::query_scalar!(
            "SELECT EXISTS (SELECT 1 FROM kolomoni.user WHERE id = $1)",
            user_id.into_inner()
        )
        .fetch_one(connection)
        .await
        .map(|exists| exists.unwrap_or(false))
        .map_err(|error| QueryError::SqlxError { error })
    }

    pub async fn exists_by_username<U>(
        connection: &mut PgConnection,
        username: U,
    ) -> QueryResult<bool>
    where
        U: AsRef<str>,
    {
        sqlx::query_scalar!(
            "SELECT EXISTS (SELECT 1 FROM kolomoni.user WHERE username = $1)",
            username.as_ref()
        )
        .fetch_one(connection)
        .await
        .map(|exists| exists.unwrap_or(false))
        .map_err(|error| QueryError::SqlxError { error })
    }

    pub async fn exists_by_display_name<U>(
        connection: &mut PgConnection,
        display_name: U,
    ) -> QueryResult<bool>
    where
        U: AsRef<str>,
    {
        sqlx::query_scalar!(
            "SELECT EXISTS (SELECT 1 FROM kolomoni.user WHERE display_name = $1)",
            display_name.as_ref()
        )
        .fetch_one(connection)
        .await
        .map(|exists| exists.unwrap_or(false))
        .map_err(|error| QueryError::SqlxError { error })
    }

    pub async fn validate_credentials<U, P>(
        connection: &mut PgConnection,
        hasher: &ArgonHasher,
        username: U,
        password: P,
    ) -> QueryResult<Option<super::Model>, UserCredentialValidationError>
    where
        U: AsRef<str>,
        P: AsRef<str>,
    {
        let potential_user = Self::get_user_by_username(connection, username).await?;

        let Some(user) = potential_user else {
            return Ok(None);
        };

        let is_valid_password = hasher
            .verify_password_against_hash(password.as_ref(), &user.hashed_password)
            .map_err(|error| UserCredentialValidationError::HasherError { error })?;


        if is_valid_password {
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    pub fn get_all_users(connection: &mut PgConnection) -> UserStream<'_> {
        let user_stream = sqlx::query_as!(
            super::IntermediateModel,
            "SELECT \
                id, username, display_name, hashed_password, \
                joined_at, last_modified_at, last_active_at \
            FROM kolomoni.user"
        )
        .fetch(connection);

        UserStream::new(user_stream)
    }
}
