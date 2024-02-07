use actix_utils::future;
use actix_utils::future::Ready;
use actix_web::dev::Payload;
use actix_web::http::{header, StatusCode};
use actix_web::web::Data;
use actix_web::{FromRequest, HttpRequest};
use kolomoni_auth::UserPermissionSet;
use kolomoni_auth::{JWTClaims, JWTValidationError};
use kolomoni_database::query::UserPermissionsExt;
use miette::{miette, Result};
use sea_orm::ConnectionTrait;
use tracing::{debug, error, info};

use crate::state::ApplicationStateInner;



/// User authentication state (actix extractor).
/// **Holding this struct doesn't automatically mean the user is authenticated!**
///
/// ## Usage with Actix
/// To easily extract authentication and permission data on an endpoint handler,
/// `UserAuth` is in reality an [Actix extractor](https://actix.rs/docs/extractors).
///
/// To use it, simply add a `user_auth: `[`UserAuth`] parameter to your endpoint handler.
/// Inside the handler body, you can then call any of e.g.
/// [`Self::token_if_authenticated`], [`Self::permissions_if_authenticated`]
/// or [`Self::token_and_permissions_if_authenticated`] that all return `Option`s and the requested
/// information, depending on your use-case.
///
/// Even better, use the following macros to further reduce boilerplate code:
/// - [`require_authentication`][crate::require_authentication] --- ensures the user is authenticated
///   and looks up the user's permissions, and
/// - [`require_permission`][crate::require_permission] --- ensures the user holds a specific permission.
///
/// See their documentation for more information and examples.
///
/// Note that getting permissions requires a database lookup.
pub enum UserAuth {
    Unauthenticated,
    Authenticated { token: JWTClaims },
}

impl UserAuth {
    /// If authenticated, return a reference to the token's contents (claims).
    #[allow(dead_code)]
    #[inline]
    pub fn token_if_authenticated(&self) -> Option<&JWTClaims> {
        match self {
            UserAuth::Unauthenticated => None,
            UserAuth::Authenticated { token } => Some(token),
        }
    }

    /// If authenticated, look up the user's permissions and return them.
    #[inline]
    pub async fn permissions_if_authenticated<C: ConnectionTrait>(
        &self,
        database: &C,
    ) -> Result<Option<UserPermissionSet>> {
        match self {
            UserAuth::Unauthenticated => Ok(None),
            UserAuth::Authenticated { token } => {
                let user_permissions =
                    UserPermissionSet::get_from_database_by_username(database, &token.username)
                        .await?;

                Ok(user_permissions)
            }
        }
    }

    /// If authenticated, return a tuple:
    /// - [`&JWTClaims`][JWTClaims] --- a reference to the token's contents, and
    /// - [`UserPermissionSet`] --- the permission list of the user.
    ///
    /// This requires a database lookup (if authenticated).
    #[inline]
    pub async fn token_and_permissions_if_authenticated<C: ConnectionTrait>(
        &self,
        database: &C,
    ) -> Result<Option<(&JWTClaims, UserPermissionSet)>> {
        match self {
            UserAuth::Unauthenticated => Ok(None),
            UserAuth::Authenticated { token } => {
                let user_permissions =
                    UserPermissionSet::get_from_database_by_username(database, &token.username)
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
                let jwt_manager = match req.app_data::<Data<ApplicationStateInner>>() {
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
