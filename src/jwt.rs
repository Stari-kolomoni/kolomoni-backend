use std::ops::Add;

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::TimestampSeconds;
use thiserror::Error;

use crate::configuration::Config;

const JWT_ISSUER: &str = "Stari Kolomoni";
const JWT_SUBJECT: &str = "API token";


#[derive(Error, Debug)]
pub enum JWTValidationError {
    #[error("token has expired")]
    Expired(JWTClaims),

    #[error("token is invalid: `{0}`")]
    InvalidToken(String),
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum JWTTokenType {
    #[serde(rename = "access")]
    Access,

    #[serde(rename = "refresh")]
    Refresh,
}

/// For more information see:
/// - https://jwt.io/introduction
/// - https://datatracker.ietf.org/doc/html/rfc7519#section-4.1
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JWTClaims {
    /// JWT registered claim: Issuer
    pub iss: String,

    /// JWT registered claim: Subject
    pub sub: String,

    /// JWT registered claim: Issued At
    #[serde_as(as = "TimestampSeconds<i64>")]
    pub iat: DateTime<Utc>,

    /// JWT registered claim: Expiration Time
    #[serde_as(as = "TimestampSeconds<i64>")]
    pub exp: DateTime<Utc>,

    /// JWT private claim: Username
    pub username: String,

    /// JWT private claim: Token type (access or refresh token)
    pub token_type: JWTTokenType,
}

impl JWTClaims {
    pub fn create(
        username: String,
        issued_at: DateTime<Utc>,
        valid_for: Duration,
        token_type: JWTTokenType,
    ) -> Self {
        let expires_on = issued_at.add(valid_for);

        Self {
            iss: JWT_ISSUER.to_string(),
            sub: JWT_SUBJECT.to_string(),
            iat: issued_at,
            exp: expires_on,
            username,
            token_type,
        }
    }
}

pub struct JsonWebTokenManager {
    header: Header,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JsonWebTokenManager {
    pub fn new(config: &Config) -> Self {
        let header = Header::new(Algorithm::HS256);
        let encoding_key = EncodingKey::from_secret(config.jsonwebtoken.secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.jsonwebtoken.secret.as_bytes());

        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[JWT_ISSUER]);
        validation.sub = Some(JWT_SUBJECT.to_string());
        validation.validate_exp = false;
        validation.validate_nbf = false;

        Self {
            header,
            encoding_key,
            decoding_key,
            validation,
        }
    }

    pub fn create_token(&self, claims: JWTClaims) -> Result<String> {
        jsonwebtoken::encode(&self.header, &claims, &self.encoding_key)
            .with_context(|| "Failed to create JWT token.")
    }

    pub fn decode_token(&self, token: &str) -> Result<JWTClaims, JWTValidationError> {
        let token_data =
            jsonwebtoken::decode::<JWTClaims>(token, &self.decoding_key, &self.validation)
                .map_err(|err| match err.kind() {
                    ErrorKind::InvalidIssuer => "Invalid token: invalid issuer.".to_string(),
                    ErrorKind::InvalidSubject => "Invalid token: invalid subject.".to_string(),
                    _ => format!("Errored while decoding token: {err}."),
                })
                .map_err(JWTValidationError::InvalidToken)?;

        let current_time = Utc::now();

        // Validate issued at (if `iat` is in the future, this token is broken)
        if token_data.claims.iat > current_time {
            return Err(JWTValidationError::InvalidToken(
                "Invalid token: `iat` field is in the future!".to_string(),
            ));
        }

        // Validate expiry time (if `exp` is in the past, it has expired)
        if token_data.claims.exp <= current_time {
            return Err(JWTValidationError::Expired(token_data.claims));
        }

        Ok(token_data.claims)
    }
}
