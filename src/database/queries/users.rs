use anyhow::{Context, Result};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

use super::super::entities::{users, users::Entity as User};
use crate::database::mutation::users::ArgonHasher;

pub struct Query {}

impl Query {
    pub async fn get_user_by_id<C: ConnectionTrait>(
        database: &C,
        id: i32,
    ) -> Result<Option<users::Model>> {
        User::find_by_id(id)
            .one(database)
            .await
            .with_context(|| "Failed to search database for user (by ID).")
    }

    pub async fn get_user_by_username<C: ConnectionTrait>(
        database: &C,
        username: &str,
    ) -> Result<Option<users::Model>> {
        User::find()
            .filter(users::Column::Username.eq(username))
            .one(database)
            .await
            .with_context(|| "Failed to search database for user (by username).")
    }

    pub async fn user_exists_by_username<C: ConnectionTrait>(
        database: &C,
        username: &str,
    ) -> Result<bool> {
        Ok(Self::get_user_by_username(database, username)
            .await?
            .is_some())
    }

    pub async fn validate_user_credentials<C: ConnectionTrait>(
        database: &C,
        hasher: &ArgonHasher,
        username: &str,
        password: &str,
    ) -> Result<bool> {
        let user = User::find()
            .filter(users::Column::Username.eq(username))
            .one(database)
            .await?;

        if let Some(user) = user {
            let is_valid_password = hasher
                .verify_password_against_hash(password, &user.hashed_password)
                .with_context(|| "Errored while validating password against hash.")?;

            Ok(is_valid_password)
        } else {
            Ok(false)
        }
    }

    pub async fn get_all_users<C: ConnectionTrait>(database: &C) -> Result<Vec<users::Model>> {
        let users = User::find().all(database).await?;

        Ok(users)
    }
}
