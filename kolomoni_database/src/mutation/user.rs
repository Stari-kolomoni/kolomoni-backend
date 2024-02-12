use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use chrono::{DateTime, Utc};
use kolomoni_auth::DEFAULT_USER_PERMISSIONS;
use kolomoni_configuration::Configuration;
use miette::{miette, Context, IntoDiagnostic, Result};
use sea_orm::{
    ActiveModelTrait,
    ActiveValue,
    ColumnTrait,
    ConnectionTrait,
    DbConn,
    EntityTrait,
    QueryFilter,
    TransactionTrait,
};

use super::super::entities::user;
use crate::entities::user_permission;
use crate::query;

pub struct ArgonHasher {
    salt_string: SaltString,
    argon_hasher: Argon2<'static>,
}

impl ArgonHasher {
    pub fn new(config: &Configuration) -> Result<Self> {
        let salt_string = SaltString::from_b64(&config.secrets.hash_salt)
            .map_err(|error| miette!("Failed to initialize SaltString: {error}."))?;

        let argon_hasher = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::default(),
        );

        Ok(Self {
            salt_string,
            argon_hasher,
        })
    }

    pub fn hash_password(&self, password: &str) -> Result<PasswordHash> {
        self.argon_hasher
            .hash_password(password.as_bytes(), &self.salt_string)
            .map_err(|error| miette!("Errored while hashing password: {error}"))
    }

    pub fn verify_password_against_hash(
        &self,
        password: &str,
        hashed_password: &str,
    ) -> Result<bool> {
        let hashed_password = PasswordHash::new(hashed_password)
            .map_err(|error| miette!("Errored while parsing hashed password: {error}"))?;

        Ok(self
            .argon_hasher
            .verify_password(password.as_bytes(), &hashed_password)
            .is_ok())
    }
}


pub struct UserRegistrationInfo {
    pub username: String,
    pub display_name: String,
    pub password: String,
}



/// Mutations for the [`crate::entities::user::Entity`] entity.
pub struct UserMutation;

impl UserMutation {
    /// Create a new user.
    pub async fn create_user(
        database: &DbConn,
        hasher: &ArgonHasher,
        registration_info: UserRegistrationInfo,
    ) -> Result<user::Model> {
        let transaction = database
            .begin()
            .await
            .into_diagnostic()
            .wrap_err("Failed to begin database transaction.")?;

        // Hash password and register user into database.
        let hashed_password = hasher
            .hash_password(&registration_info.password)
            .wrap_err("Failed to hash password.")?;

        let registration_time = Utc::now();

        let user = user::ActiveModel {
            username: ActiveValue::Set(registration_info.username),
            display_name: ActiveValue::Set(registration_info.display_name),
            hashed_password: ActiveValue::Set(hashed_password.to_string()),
            joined_at: ActiveValue::Set(registration_time.fixed_offset()),
            last_modified_at: ActiveValue::Set(registration_time.fixed_offset()),
            last_active_at: ActiveValue::Set(registration_time.fixed_offset()),
            ..Default::default()
        }
        .insert(&transaction)
        .await
        .into_diagnostic()
        .wrap_err("Failed to save user into database.")?;

        // Give the default permissions to the new user.
        for permission in DEFAULT_USER_PERMISSIONS {
            user_permission::ActiveModel {
                user_id: ActiveValue::Set(user.id),
                permission_id: ActiveValue::Set(permission.id()),
            }
            .insert(&transaction)
            .await
            .into_diagnostic()
            .wrap_err("Failed to add default permissions to user.")?;
        }

        transaction
            .commit()
            .await
            .into_diagnostic()
            .wrap_err("Failed to commit transaction.")?;

        Ok(user)
    }

    /// Update last activity time for a user. The user is looked up by their username.
    pub async fn update_last_active_at_by_username<C: ConnectionTrait>(
        database: &C,
        username: &str,
        last_active_at: Option<DateTime<Utc>>,
    ) -> Result<user::Model> {
        // TODO This can be further optimized by using a lower-level query.

        let user = query::UserQuery::get_user_by_username(database, username)
            .await?
            .ok_or_else(|| miette!("Invalid username, no such user."))?;

        let user_with_updated_last_activity = user::ActiveModel {
            id: ActiveValue::Unchanged(user.id),
            last_active_at: ActiveValue::Set(last_active_at.unwrap_or_else(Utc::now).fixed_offset()),
            ..Default::default()
        };

        let updated_user = user_with_updated_last_activity
            .update(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while updating a user's last activity time (by username).")?;

        Ok(updated_user)
    }

    /// Update last activity time for a user. The user is looked up by their ID.
    pub async fn update_last_active_at_by_user_id<C: ConnectionTrait>(
        database: &C,
        user_id: i32,
        last_active_at: Option<DateTime<Utc>>,
    ) -> Result<user::Model> {
        let user_with_updated_last_activity = user::ActiveModel {
            id: ActiveValue::Unchanged(user_id),
            last_active_at: ActiveValue::Set(last_active_at.unwrap_or_else(Utc::now).fixed_offset()),
            ..Default::default()
        };

        let updated_user = user_with_updated_last_activity
            .update(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while updating a user's last activity time (by ID).")?;

        Ok(updated_user)
    }

    /// Update a user's display name. The user is looked up by their ID.
    pub async fn update_display_name_by_user_id<C: ConnectionTrait>(
        database: &C,
        user_id: i32,
        new_display_name: String,
    ) -> Result<user::Model> {
        let user_with_updated_display_name = user::ActiveModel {
            id: ActiveValue::Unchanged(user_id),
            display_name: ActiveValue::Set(new_display_name),
            last_modified_at: ActiveValue::Set(Utc::now().fixed_offset()),
            ..Default::default()
        };

        user_with_updated_display_name
            .update(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while updating a user's display name in the database (by ID).")?;

        let updated_user = query::UserQuery::get_user_by_id(database, user_id)
            .await?
            .ok_or_else(|| miette!("BUG: No such user ID: {user_id}"))?;

        Ok(updated_user)
    }

    /// Update a user's display name. The user is looked up by their username.
    pub async fn update_display_name_by_username<C: ConnectionTrait>(
        database: &C,
        username: &str,
        new_display_name: String,
    ) -> Result<user::Model> {
        let current_time = Utc::now().fixed_offset();

        let user_with_updated_display_name = user::ActiveModel {
            display_name: ActiveValue::Set(new_display_name),
            last_active_at: ActiveValue::Set(current_time),
            last_modified_at: ActiveValue::Set(current_time),
            ..Default::default()
        };

        let result = user::Entity::update_many()
            .set(user_with_updated_display_name)
            .filter(user::Column::Username.eq(username))
            .exec(database)
            .await
            .into_diagnostic()
            .wrap_err(
                "Failed while updating a user's display name in the database (by username).",
            )?;

        if result.rows_affected != 1 {
            return Err(miette!(
                "BUG: Updated {} rows instead of 1!",
                result.rows_affected
            ));
        }

        let updated_user = query::UserQuery::get_user_by_username(database, username)
            .await?
            .ok_or_else(|| miette!("BUG: No such user: {username}"))?;

        Ok(updated_user)
    }
}
