use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use thiserror::Error;


#[derive(Debug, Error)]
pub enum ArgonHasherError {
    #[error("argon2 error: {}", .error)]
    Argon2Error { error: argon2::password_hash::Error },
}



/// TODO refactor to "password hasher"
pub struct ArgonHasher {
    salt_string: SaltString,
    argon_hasher: Argon2<'static>,
}

impl ArgonHasher {
    pub fn new(base64_hash_salt: &str) -> Result<Self, ArgonHasherError> {
        let salt_string = SaltString::from_b64(base64_hash_salt)
            .map_err(|error| ArgonHasherError::Argon2Error { error })?;

        let argon_hasher = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::default(),
        );

        Ok(Self {
            salt_string,
            argon_hasher,
        })
    }

    pub fn hash_password(&self, password: &str) -> Result<PasswordHash, ArgonHasherError> {
        self.argon_hasher
            .hash_password(password.as_bytes(), &self.salt_string)
            .map_err(|error| ArgonHasherError::Argon2Error { error })
    }

    pub fn verify_password_against_hash(
        &self,
        password: &str,
        hashed_password: &str,
    ) -> Result<bool, ArgonHasherError> {
        let hashed_password = PasswordHash::new(hashed_password)
            .map_err(|error| ArgonHasherError::Argon2Error { error })?;

        Ok(self
            .argon_hasher
            .verify_password(password.as_bytes(), &hashed_password)
            .is_ok())
    }
}
