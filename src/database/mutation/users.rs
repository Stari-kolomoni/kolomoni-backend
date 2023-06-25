use anyhow::{anyhow, Context, Result};
use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue, DbConn};

use super::super::entities::users;
use crate::configuration::Config;

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

pub struct Mutation {}

impl Mutation {
    pub async fn create_user(
        database: &DbConn,
        hasher: &ArgonHasher,
        registration_info: UserRegistrationInfo,
    ) -> Result<users::ActiveModel> {
        // TODO Default permissions?

        let hashed_password = hasher
            .hash_password(&registration_info.password)
            .with_context(|| "Failed to hash password.")?;

        let registration_time = Utc::now();

        users::ActiveModel {
            username: ActiveValue::Set(registration_info.username),
            display_name: ActiveValue::Set(registration_info.display_name),
            hashed_password: ActiveValue::Set(hashed_password.to_string()),
            joined_at: ActiveValue::Set(registration_time),
            last_modified_at: ActiveValue::Set(registration_time),
            last_active_at: ActiveValue::Set(registration_time),
            ..Default::default()
        }
        .save(database)
        .await
        .map_err(|err| {
            anyhow!(
                "Failed to save user into database: {}",
                err.to_string()
            )
        })
    }
}
