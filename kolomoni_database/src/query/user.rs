use miette::{Context, IntoDiagnostic, Result};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

use super::super::entities::prelude::User;
use super::super::entities::user;
use crate::mutation::ArgonHasher;


/// Queries related to the [`crate::entities::user::Entity`] entity.
pub struct UserQuery;

impl UserQuery {
    /// Get a user by their ID.
    pub async fn get_user_by_id<C: ConnectionTrait>(
        database: &C,
        id: i32,
    ) -> Result<Option<user::Model>> {
        User::find_by_id(id)
            .one(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while searching database for user by ID.")
    }

    /// Get a user by their username.
    pub async fn get_user_by_username<C: ConnectionTrait>(
        database: &C,
        username: &str,
    ) -> Result<Option<user::Model>> {
        User::find()
            .filter(user::Column::Username.eq(username))
            .one(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while searching database for user by username.")
    }

    /// Check whether a user exists (by their username).
    pub async fn user_exists_by_username<C: ConnectionTrait>(
        database: &C,
        username: &str,
    ) -> Result<bool> {
        Ok(Self::get_user_by_username(database, username)
            .await?
            .is_some())
    }

    /// Check whether a user exists (by their display name).
    pub async fn user_exists_by_display_name<C: ConnectionTrait>(
        database: &C,
        display_name: &str,
    ) -> Result<bool> {
        let user = User::find()
            .filter(user::Column::DisplayName.eq(display_name))
            .one(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while looking up whether user exists by username.")?;

        Ok(user.is_some())
    }

    /// Validate a user's credentials (the username and password combination).
    /// This is basically the login verification method.
    pub async fn validate_user_credentials<C: ConnectionTrait>(
        database: &C,
        hasher: &ArgonHasher,
        username: &str,
        password: &str,
    ) -> Result<bool> {
        let user = User::find()
            .filter(user::Column::Username.eq(username))
            .one(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while looking up user in database (by username).")?;

        if let Some(user) = user {
            let is_valid_password = hasher
                .verify_password_against_hash(password, &user.hashed_password)
                .wrap_err("Errored while validating password against hash.")?;

            Ok(is_valid_password)
        } else {
            Ok(false)
        }
    }

    /// Get a list of all registered users.
    pub async fn get_all_users<C: ConnectionTrait>(database: &C) -> Result<Vec<user::Model>> {
        let users = User::find()
            .all(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while querying all users from database.")?;

        Ok(users)
    }
}
