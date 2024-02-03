use actix_utils::future;
use actix_utils::future::Ready;
use actix_web::dev::Payload;
use actix_web::http::{header, StatusCode};
use actix_web::web::Data;
use actix_web::{FromRequest, HttpRequest};
use kolomoni_auth::permissions::UserPermissions;
use kolomoni_auth::token::{JWTClaims, JWTValidationError};
use kolomoni_database::query::UserPermissionsExt;
use miette::{miette, Result};
use sea_orm::ConnectionTrait;
use tracing::{debug, error, info};

use crate::state::AppState;



/// User authentication state. Holding this struct doesn't automatically mean
/// the user is authenticated (see the enum variants).
///
/// ## Use in actix-web
/// To easily extract authentication and permission data on an endpoint handler,
/// `UserAuth` can be an extractor (https://actix.rs/docs/extractors). Simply
/// add a `user_auth: UserAuth` parameter to your endpoint handler and that's it.
///
/// Inside the handler body, you can call any of `token_if_authenticated`, `permissions_if_authenticated`
/// or `token_and_permissions_if_authenticated` that all return `Option`s and the requested
/// information, depending on your use-case.
///
/// Note that getting permissions requires a database lookup.
pub enum UserAuth {
    Unauthenticated,
    Authenticated { token: JWTClaims },
}

impl UserAuth {
    /// If authenticated, return `Some` containing a reference to the token contents.
    #[allow(dead_code)]
    #[inline]
    pub fn token_if_authenticated(&self) -> Option<&JWTClaims> {
        match self {
            UserAuth::Unauthenticated => None,
            UserAuth::Authenticated { token } => Some(token),
        }
    }

    /// If authenticated, lookup permissions for the user and return `Some`
    /// containing `UserPermissions`.
    #[inline]
    pub async fn permissions_if_authenticated<C: ConnectionTrait>(
        &self,
        database: &C,
    ) -> Result<Option<UserPermissions>> {
        match self {
            UserAuth::Unauthenticated => Ok(None),
            UserAuth::Authenticated { token } => {
                let user_permissions =
                    UserPermissions::get_from_database_by_username(database, &token.username)
                        .await?;

                Ok(user_permissions)
            }
        }
    }

    /// If authenticated, return `Some` containing a tuple of:
    /// - a reference to the token contents (`&JWTClaims`) and
    /// - `UserPermissions`, which is the permission list of the user.
    ///
    /// This requires a database lookup (if authenticated).
    #[inline]
    pub async fn token_and_permissions_if_authenticated<C: ConnectionTrait>(
        &self,
        database: &C,
    ) -> Result<Option<(&JWTClaims, UserPermissions)>> {
        match self {
            UserAuth::Unauthenticated => Ok(None),
            UserAuth::Authenticated { token } => {
                let user_permissions =
                    UserPermissions::get_from_database_by_username(database, &token.username)
                        .await?
                        .ok_or_else(|| miette!("User is missing from the database."))?;

                Ok(Some((token, user_permissions)))
            }
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
                            actix_web::error::InternalError::new(
                                "Missing AppState.",
                                StatusCode::INTERNAL_SERVER_ERROR,
                            )
                            .into(),
                        );
                    }
                };

                let header_value = match authorization_header_value.to_str() {
                    Ok(header_value) => header_value,
                    Err(_) => return future::err(actix_web::error::ParseError::Header.into()),
                };

                // Strip Bearer prefix
                if !header_value.starts_with("Bearer ") {
                    return future::err(actix_web::error::ParseError::Header.into());
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

                                future::err(actix_web::error::ErrorForbidden(
                                    "Authentication token expired.",
                                ))
                            }
                            JWTValidationError::InvalidToken(error) => {
                                info!(
                                    error = error,
                                    "User tried authenticating with invalid token."
                                );

                                future::err(actix_web::error::ErrorBadRequest(
                                    "Invalid token.",
                                ))
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
