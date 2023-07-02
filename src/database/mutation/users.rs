use anyhow::{anyhow, Context, Result};
use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use chrono::{DateTime, Utc};
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

use super::super::entities::users;
use crate::auth::permissions::DEFAULT_USER_PERMISSIONS;
use crate::configuration::Config;
use crate::database::{entities, query};

pub struct ArgonHasher {
    salt_string: SaltString,
    argon_hasher: Argon2<'static>,
}

impl ArgonHasher {
    pub fn new(config: &Config) -> Result<Self> {
        let salt_string = SaltString::from_b64(&config.password.hash_salt)
            .map_err(|err| anyhow!("Errored while initializing salt: {err}"))?;

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
            .map_err(|err| anyhow!("Errored while hashing password: {err}"))
    }

    pub fn verify_password_against_hash(
        &self,
        password: &str,
        hashed_password: &str,
    ) -> Result<bool> {
        let hashed_password = PasswordHash::new(hashed_password)
            .map_err(|err| anyhow!("Errored while parsing hashed password: {err}"))?;

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

pub struct UsersMutation {}

impl UsersMutation {
    pub async fn create_user(
        database: &DbConn,
        hasher: &ArgonHasher,
        registration_info: UserRegistrationInfo,
    ) -> Result<users::Model> {
        let transaction = database.begin().await?;

        // Hash password and register user into database.
        let hashed_password = hasher
            .hash_password(&registration_info.password)
            .with_context(|| "Failed to hash password.")?;

        let registration_time = Utc::now();

        let user = users::ActiveModel {
            username: ActiveValue::Set(registration_info.username),
            display_name: ActiveValue::Set(registration_info.display_name),
            hashed_password: ActiveValue::Set(hashed_password.to_string()),
            joined_at: ActiveValue::Set(registration_time),
            last_modified_at: ActiveValue::Set(registration_time),
            last_active_at: ActiveValue::Set(registration_time),
            ..Default::default()
        }
        .insert(&transaction)
        .await
        .map_err(|err| anyhow!("Failed to save user into database: {err}"))?;

        // Give the default permissions to the new user.
        for permission in DEFAULT_USER_PERMISSIONS {
            entities::user_permissions::ActiveModel {
                user_id: ActiveValue::Set(user.id),
                permission_id: ActiveValue::Set(permission.to_id()),
            }
            .insert(&transaction)
            .await
            .map_err(|err| anyhow!("Failed to add default permission to user: {err}"))?;
        }

        transaction.commit().await?;

        Ok(user)
    }

    pub async fn update_last_active_at_by_username<C: ConnectionTrait>(
        database: &C,
        username: &str,
        last_active_at: Option<DateTime<Utc>>,
    ) -> Result<users::Model> {
        // TODO This can be further optimized by using a lower-level query.

        let user = query::UsersQuery::get_user_by_username(database, username)
            .await?
            .ok_or_else(|| anyhow!("Invalid username, no such user."))?;

        let user_with_updated_last_activity = users::ActiveModel {
            id: ActiveValue::Unchanged(user.id),
            last_active_at: ActiveValue::Set(last_active_at.unwrap_or_else(Utc::now)),
            ..Default::default()
        };

        let updated_user = user_with_updated_last_activity.update(database).await?;

        Ok(updated_user)
    }

    pub async fn update_last_active_at_by_user_id<C: ConnectionTrait>(
        database: &C,
        user_id: i32,
        last_active_at: Option<DateTime<Utc>>,
    ) -> Result<users::Model> {
        let user_with_updated_last_activity = users::ActiveModel {
            id: ActiveValue::Unchanged(user_id),
            last_active_at: ActiveValue::Set(last_active_at.unwrap_or_else(Utc::now)),
            ..Default::default()
        };

        let updated_user = user_with_updated_last_activity.update(database).await?;

        Ok(updated_user)
    }

    pub async fn update_display_name_by_user_id<C: ConnectionTrait>(
        database: &C,
        user_id: i32,
        new_display_name: String,
    ) -> Result<users::Model> {
        let user_with_updated_display_name = users::ActiveModel {
            id: ActiveValue::Unchanged(user_id),
            display_name: ActiveValue::Set(new_display_name),
            last_modified_at: ActiveValue::Set(Utc::now()),
            ..Default::default()
        };

        user_with_updated_display_name.update(database).await?;

        let updated_user = query::UsersQuery::get_user_by_id(database, user_id)
            .await?
            .ok_or_else(|| anyhow!("BUG: No such user ID: {user_id}"))?;

        Ok(updated_user)
    }

    pub async fn update_display_name_by_username<C: ConnectionTrait>(
        database: &C,
        username: &str,
        new_display_name: String,
    ) -> Result<users::Model> {
        let user_with_updated_display_name = users::ActiveModel {
            display_name: ActiveValue::Set(new_display_name),
            last_modified_at: ActiveValue::Set(Utc::now()),
            ..Default::default()
        };

        let result = users::Entity::update_many()
            .set(user_with_updated_display_name)
            .filter(users::Column::Username.eq(username))
            .exec(database)
            .await?;

        if result.rows_affected != 1 {
            return Err(anyhow!(
                "BUG: Updated {} rows instead of 1!",
                result.rows_affected
            ));
        }

        let updated_user = query::UsersQuery::get_user_by_username(database, username)
            .await?
            .ok_or_else(|| anyhow!("BUG: No such user: {username}"))?;

        Ok(updated_user)
    }
}
