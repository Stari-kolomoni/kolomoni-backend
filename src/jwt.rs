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


/// JSON Web Token validation error type.
/// A token can be either expired or simply invalid.
#[derive(Error, Debug)]
pub enum JWTValidationError {
    #[error("token has expired")]
    Expired(JWTClaims),

    #[error("token is invalid: `{0}`")]
    InvalidToken(String),
}

/// Type of one of our JSON Web Tokens.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum JWTTokenType {
    #[serde(rename = "access")]
    Access,

    #[serde(rename = "refresh")]
    Refresh,
}

/// JSON Web Token data ("claims").
/// Can be either an access token or a refresh token.
///
/// For more information see:
/// - https://jwt.io/introduction
/// - https://datatracker.ietf.org/doc/html/rfc7519#section-4.1
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JWTClaims {
    /// JWT registered claim: Issuer
    ///
    /// Should always be the same as `JWT_ISSUER`.
    pub iss: String,

    /// JWT registered claim: Subject
    ///
    /// Should always be the same as `JWT_SUBJECT`.
    pub sub: String,

    /// JWT registered claim: Issued At
    #[serde_as(as = "TimestampSeconds<i64>")]
    pub iat: DateTime<Utc>,

    /// JWT registered claim: Expiration Time
    #[serde_as(as = "TimestampSeconds<i64>")]
    pub exp: DateTime<Utc>,

    /// JWT private claim: Username
    ///
    /// Username of the user that this token belongs to.
    pub username: String,

    /// JWT private claim: Token type (access or refresh token)
    ///
    /// Access tokens can be used to call restricted endpoints and
    /// refresh tokens can be used to generate new access tokens when they
    /// expire (refresh tokens have a longer expiration time).
    pub token_type: JWTTokenType,
}

impl JWTClaims {
    /// Create a new JSON Web Token.
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

/// Central JSON Web Token manager (encoder and decoder).
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

        // Validate issuer and subject automatically when decoding.
        validation.set_issuer(&[JWT_ISSUER]);
        validation.sub = Some(JWT_SUBJECT.to_string());

        // Disable "expiry" and "not before" validation, we'll do it ourselves (as we use `chrono`).
        validation.validate_exp = false;
        validation.validate_nbf = false;

        Self {
            header,
            encoding_key,
            decoding_key,
            validation,
        }
    }

    /// Create (encode) a new token into a string.
    pub fn create_token(&self, claims: JWTClaims) -> Result<String> {
        jsonwebtoken::encode(&self.header, &claims, &self.encoding_key)
            .with_context(|| "Failed to create JWT token.")
    }

    /// Decode a token from a string.
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
