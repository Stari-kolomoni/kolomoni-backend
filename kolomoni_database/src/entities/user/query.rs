use futures_core::stream::BoxStream;
use kolomoni_core::{ids::UserId, password_hasher::ArgonHasher};
use sqlx::PgConnection;

use super::{UserQueryError, UserQueryResult};
use crate::{IntoExternalModel, QueryError, QueryResult};




type RawUserStream<'c> = BoxStream<'c, Result<super::InternalUserModel, sqlx::Error>>;

create_async_stream_wrapper!(
    pub struct UserStream<'c>;
    transforms stream RawUserStream<'c> => stream of QueryResult<super::UserModel>:
        |value|
            value.map(|result| {
                result
                    .map(super::InternalUserModel::into_external_model)
                    .map_err(|error| QueryError::SqlxError { error })
            })
);




pub struct UserQuery;

impl UserQuery {
    pub async fn get_user_by_id(
        connection: &mut PgConnection,
        user_id: UserId,
    ) -> QueryResult<Option<super::UserModel>> {
        let optional_intermediate_model = sqlx::query_as!(
            super::InternalUserModel,
            "SELECT \
                    id, username, display_name, hashed_password, \
                    joined_at, last_modified_at, last_active_at \
                FROM kolomoni.user \
                WHERE id = $1",
            user_id.into_uuid()
        )
        .fetch_optional(connection)
        .await?;

        Ok(optional_intermediate_model.map(super::InternalUserModel::into_external_model))
    }

    pub async fn get_user_by_username<U>(
        connection: &mut PgConnection,
        username: U,
    ) -> QueryResult<Option<super::UserModel>>
    where
        U: AsRef<str>,
    {
        let optional_intermediate_model = sqlx::query_as!(
            super::InternalUserModel,
            "SELECT \
                    id, username, display_name, hashed_password, \
                    joined_at, last_modified_at, last_active_at \
                FROM kolomoni.user \
                WHERE username = $1",
            username.as_ref()
        )
        .fetch_optional(connection)
        .await?;

        Ok(optional_intermediate_model.map(super::InternalUserModel::into_external_model))
    }

    pub async fn exists_by_id(connection: &mut PgConnection, user_id: UserId) -> QueryResult<bool> {
        sqlx::query_scalar!(
            "SELECT EXISTS (SELECT 1 FROM kolomoni.user WHERE id = $1)",
            user_id.into_uuid()
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
    ) -> UserQueryResult<Option<super::UserModel>>
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
            .map_err(|error| UserQueryError::HasherError { error })?;


        if is_valid_password {
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    pub fn get_all_users(connection: &mut PgConnection) -> UserStream<'_> {
        let user_stream = sqlx::query_as!(
            super::InternalUserModel,
            "SELECT \
                    id, username, display_name, hashed_password, \
                    joined_at, last_modified_at, last_active_at \
                FROM kolomoni.user"
        )
        .fetch(connection);

        UserStream::new(user_stream)
    }
}
