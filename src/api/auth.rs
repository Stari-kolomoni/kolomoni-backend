// TODO JWT Bearer token extractor (not middleware!), see
//      https://actix.rs/docs/extractors

use actix_utils::future;
use actix_utils::future::Ready;
use actix_web::dev::Payload;
use actix_web::http::{header, StatusCode};
use actix_web::web::Data;
use actix_web::{error, FromRequest, HttpRequest};
use tracing::{debug, error, info};

use crate::jwt::{JWTClaims, JWTValidationError};
use crate::state::AppState;

pub enum UserAuth {
    Unauthenticated,
    Authenticated { token: JWTClaims },
}

// TODO additional info, including permissions
impl UserAuth {
    #[inline]
    pub fn auth_token(&self) -> Option<&JWTClaims> {
        match self {
            UserAuth::Unauthenticated => None,
            UserAuth::Authenticated { token } => Some(token),
        }
    }
}

impl FromRequest for UserAuth {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        match req.headers().get(header::AUTHORIZATION) {
            Some(authorization_header_value) => {
                let jwt_manager = match req.app_data::<Data<AppState>>() {
                    Some(app_state) => &app_state.jwt_manager,
                    None => {
                        error!("BUG: No AppState injected, all UserAuth extractors will fail!");

                        return future::err(
                            error::InternalError::new(
                                "Missing AppState.",
                                StatusCode::INTERNAL_SERVER_ERROR,
                            )
                            .into(),
                        );
                    }
                };

                let header_value = match authorization_header_value.to_str() {
                    Ok(header_value) => header_value,
                    Err(_) => return future::err(error::ParseError::Header.into()),
                };

                // Strip Bearer prefix
                if !header_value.starts_with("Bearer ") {
                    return future::err(error::ParseError::Header.into());
                }

                let token_string = header_value
                    .strip_prefix("Bearer ")
                    .expect("BUG: String started with \"Bearer \", but couldn't strip prefix.");

                let token = match jwt_manager.decode_token(token_string) {
                    Ok(token) => token,
                    Err(error) => {
                        return match error {
                            JWTValidationError::Expired(token) => {
                                debug!(
                                    username = token.username,
                                    "User tried authenticating with expired token."
                                );

                                future::err(error::ErrorForbidden(
                                    "Authentication token expired.",
                                ))
                            }
                            JWTValidationError::InvalidToken(error) => {
                                info!(
                                    error = error,
                                    "User tried authenticating with invalid token."
                                );

                                future::err(error::ErrorBadRequest("Invalid token."))
                            }
                        }
                    }
                };

                future::ok(Self::Authenticated { token })
            }
            None => future::ok(Self::Unauthenticated),
        }
    }
}
