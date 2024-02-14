use actix_utils::future;
use actix_utils::future::Ready;
use actix_web::dev::Payload;
use actix_web::http::{header, StatusCode};
use actix_web::web::Data;
use actix_web::{FromRequest, HttpRequest};
use chrono::{DateTime, Utc};
use kolomoni_auth::{JWTClaims, JWTValidationError, RoleSet, BLANKET_ANY_USER_PERMISSION_GRANT};
use kolomoni_auth::{Permission, PermissionSet};
use kolomoni_database::query::UserRoleQuery;
use miette::{Context, Result};
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
///
/// FIXME The documentation below this is outdated, update.
///
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
pub enum UserAuthenticationExtractor {
    Unauthenticated,
    Authenticated { token: JWTClaims },
}

impl UserAuthenticationExtractor {
    pub fn authenticated_user(&self) -> Option<AuthenticatedUser> {
        match self {
            UserAuthenticationExtractor::Unauthenticated => None,
            UserAuthenticationExtractor::Authenticated { token } => Some(AuthenticatedUser {
                token: token.clone(),
            }),
        }
    }
}

impl FromRequest for UserAuthenticationExtractor {
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
                                    user_id = token.user_id,
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



pub struct AuthenticatedUser {
    token: JWTClaims,
}

impl AuthenticatedUser {
    #[allow(dead_code)]
    pub fn logged_in_at(&self) -> &DateTime<Utc> {
        &self.token.iat
    }

    #[allow(dead_code)]
    pub fn login_expires_at(&self) -> &DateTime<Utc> {
        &self.token.exp
    }

    pub fn user_id(&self) -> i32 {
        self.token.user_id
    }

    /// Returns a permission set of this user.
    /// This requires one round-trip to the database.
    ///
    /// Prefer using [`Self::has_permission`] if you'll be checking for a single permission,
    /// and this method if you're checking for multiple or doing advanced permission logic.
    pub async fn permissions<C: ConnectionTrait>(&self, database: &C) -> Result<PermissionSet> {
        let permission_set =
            UserRoleQuery::effective_user_permissions_from_user_id(database, self.token.user_id)
                .await
                .wrap_err("Could not query effective permissions for user.")?;

        Ok(permission_set)
    }

    /// Returns a boolean indicating whether the authenticated user has the provided permission.
    pub async fn has_permission<C: ConnectionTrait>(
        &self,
        database: &C,
        permission: Permission,
    ) -> Result<bool> {
        if BLANKET_ANY_USER_PERMISSION_GRANT.contains(&permission) {
            return Ok(true);
        }

        UserRoleQuery::user_has_permission(database, self.token.user_id, permission)
            .await
            .wrap_err("Could not query whether the user has a specific permission.")
    }

    pub async fn roles<C: ConnectionTrait>(&self, database: &C) -> Result<RoleSet> {
        let role_set = UserRoleQuery::user_roles(database, self.token.user_id)
            .await
            .wrap_err("Could not query roles for user.")?;

        Ok(role_set)
    }
}
