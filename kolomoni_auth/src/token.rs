use std::borrow::Cow;
use std::ops::Add;

use chrono::{DateTime, Duration, SubsecRound, Utc};
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use kolomoni_core::id::UserId;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::TimestampSeconds;
use thiserror::Error;


// TODO Consider making this dynamic (for example through an environment variable).

/// JSON Web Token issuer.
const JWT_ISSUER: &str = "Stari Kolomoni";

/// JSON Web Token subject.
const JWT_SUBJECT: &str = "API token";


/// JSON Web Token validation error type.
/// A token can be either expired or simply invalid.
#[derive(Error, Debug)]
pub enum JWTValidationError {
    #[error("token has expired")]
    Expired { expired_token: JWTClaims },

    #[error("token is invalid: {}", .reason)]
    InvalidToken { reason: Cow<'static, str> },
}


/// Type of one of our JSON Web Tokens, meaning either an access or a refresh token.
///
/// Access tokens can be used to authenticate on some API request,
/// and refresh tokens can be used to obtain a new access token.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum JWTTokenType {
    /// Access token.
    #[serde(rename = "access")]
    Access,

    /// Refresh token.
    #[serde(rename = "refresh")]
    Refresh,
}


/// JSON Web Token data (also called "claims").
///
/// Can be either an access token or a refresh token.
///
/// More information:
/// - <https://jwt.io/introduction>
/// - <https://datatracker.ietf.org/doc/html/rfc7519#section-4.1>
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

    /// JWT private claim: UUIDv7 of the user the token belongs to.
    pub user_id: UserId,

    /// JWT private claim: Token type (access or refresh token)
    ///
    /// *Access tokens* can be used to call restricted endpoints.
    ///
    /// *Refresh tokens* can be used to generate new access tokens when they
    /// expire (refresh tokens have a longer expiration time compared to access tokens).
    pub token_type: JWTTokenType,
}

impl JWTClaims {
    /// Create a new JSON Web Token.
    ///
    /// Note that the `issued_at` timestamp will have its sub-second content truncated
    /// (see [`trunc_subsecs`][chrono::round::SubsecRound::trunc_subsecs]).
    pub fn create(
        user_id: UserId,
        issued_at: DateTime<Utc>,
        valid_for: Duration,
        token_type: JWTTokenType,
    ) -> Self {
        let issued_at = issued_at.trunc_subsecs(0);
        let expires_on = issued_at.add(valid_for);

        Self {
            iss: JWT_ISSUER.to_string(),
            sub: JWT_SUBJECT.to_string(),
            iat: issued_at,
            exp: expires_on,
            user_id,
            token_type,
        }
    }
}



#[derive(Debug, Error)]
pub enum JWTCreationError {
    #[error("JWT error")]
    JWTError {
        #[from]
        #[source]
        error: jsonwebtoken::errors::Error,
    },
}


/// JSON Web Token manager --- encoder and decoder.
pub struct JsonWebTokenManager {
    /// Token header.
    header: Header,

    /// JSON Web Token encoding key, derived from the provided secret.
    encoding_key: EncodingKey,

    /// JSON Web Token decoding key, derived from the provided secret.
    decoding_key: DecodingKey,

    /// A token subject and issuer validator.
    validation: Validation,
}

impl JsonWebTokenManager {
    pub fn new(json_web_token_secret: &str) -> Self {
        let header = Header::new(Algorithm::HS256);
        let encoding_key = EncodingKey::from_secret(json_web_token_secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(json_web_token_secret.as_bytes());

        let mut validation = Validation::new(Algorithm::HS256);

        // Validate issuer and subject automatically when decoding.
        validation.set_issuer(&[JWT_ISSUER]);
        validation.sub = Some(JWT_SUBJECT.to_string());

        // Disable "expiry" and "not before" validation, we'll do it ourselves
        // (we use `chrono`, which this doesn't support).
        validation.validate_exp = false;
        validation.validate_nbf = false;

        Self {
            header,
            encoding_key,
            decoding_key,
            validation,
        }
    }

    /// Create (encode) a new token. Returns a string with the encoded content.
    pub fn create_token(&self, claims: JWTClaims) -> Result<String, JWTCreationError> {
        jsonwebtoken::encode(&self.header, &claims, &self.encoding_key)
            .map_err(|error| JWTCreationError::JWTError { error })
    }

    /// Decode a JSON Web Token from a string.
    pub fn decode_token(&self, token: &str) -> Result<JWTClaims, JWTValidationError> {
        let token_data = jsonwebtoken::decode::<JWTClaims>(
            token,
            &self.decoding_key,
            &self.validation,
        )
        .map_err(|error| JWTValidationError::InvalidToken {
            reason: match error.kind() {
                ErrorKind::InvalidIssuer => Cow::from("failed to parse JWT token: invalid issuer"),
                ErrorKind::InvalidSubject => Cow::from("failed to parse JWT token: invalid subject"),
                _ => Cow::from(format!("failed to parse JWT token: {}", error)),
            },
        })?;

        let current_time = Utc::now();

        // Validate issued at (if `iat` is in the future, this token is broken)
        if token_data.claims.iat > current_time {
            return Err(JWTValidationError::InvalidToken {
                reason: Cow::from("invalid JWT token: issued-at field is in the future"),
            });
        }

        // Validate expiry time (if `exp` is in the past, it has expired)
        if token_data.claims.exp <= current_time {
            return Err(JWTValidationError::Expired {
                expired_token: token_data.claims,
            });
        }

        Ok(token_data.claims)
    }
}


#[cfg(test)]
mod test {
    use chrono::SubsecRound;
    use uuid::Uuid;

    use super::*;

    #[test]
    fn create_and_validate_token() {
        let manager = JsonWebTokenManager::new("secret");

        let issued_at = Utc::now().trunc_subsecs(0);
        let valid_for = chrono::Duration::from_std(std::time::Duration::from_secs(60)).unwrap();

        let user_id = UserId::new(Uuid::now_v7());

        let claims = JWTClaims::create(
            user_id,
            issued_at,
            valid_for,
            JWTTokenType::Access,
        );

        let encoded_token = manager.create_token(claims).unwrap();


        let decoded_claims = manager.decode_token(&encoded_token).unwrap();

        assert_eq!(decoded_claims.iss, JWT_ISSUER);
        assert_eq!(decoded_claims.sub, JWT_SUBJECT);
        assert_eq!(decoded_claims.iat, issued_at);
        assert_eq!(decoded_claims.exp, issued_at + valid_for);
        assert_eq!(decoded_claims.user_id, user_id);
        assert_eq!(decoded_claims.token_type, JWTTokenType::Access);
    }
}
