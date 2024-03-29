//! Authentication-related code.

use actix_utils::future;
use actix_utils::future::Ready;
use actix_web::dev::Payload;
use actix_web::http::{header, StatusCode};
use actix_web::web::Data;
use actix_web::{FromRequest, HttpRequest};
use chrono::{DateTime, Utc};
use kolomoni_auth::{JWTClaims, JWTValidationError, RoleSet, BLANKET_PERMISSION_GRANT};
use kolomoni_auth::{Permission, PermissionSet};
use kolomoni_database::query::UserRoleQuery;
use miette::{Context, Result};
use sea_orm::ConnectionTrait;
use tracing::{debug, error, info};

use crate::state::ApplicationStateInner;


/// User authentication extractor.
///
/// **Holding this struct doesn't automatically mean the user is authenticated!**
///
/// # Usage with Actix
/// To easily extract authentication data on an endpoint function,
/// [`UserAuthenticationExtractor`] is actually an [Actix extractor](https://actix.rs/docs/extractors).
///
/// To use it, simply add a `authentication: `[`UserAuthenticationExtractor`] parameter
/// to your endpoint function parameters.
///
/// Then, inside the handler body, you can all e.g. [`UserAuthenticationExtractor::authenticated_user`]
/// to get an `Option<`[`AuthenticatedUser`]`>`. In reality, you may want to use the
/// [`require_authentication`][crate::require_authentication] macro that directly returns
/// an [`AuthenticatedUser`], early-returning from the function with a `401 Unauthorized`
/// if the caller did not provide authentication.
///
/// See documentation of [`require_authentication`][crate::require_authentication]
/// for usage examples.
pub enum UserAuthenticationExtractor {
    /// No user authentication provided.
    Unauthenticated,

    /// Valid JWT token provided as authentication.
    Authenticated { token: JWTClaims },
}

impl UserAuthenticationExtractor {
    /// Returns an `Some(`[`AuthenticatedUser`]`)` if the API caller
    /// provided a JWT authentication token with the request.
    pub fn authenticated_user(&self) -> Option<AuthenticatedUser> {
        match self {
            UserAuthenticationExtractor::Unauthenticated => None,
            UserAuthenticationExtractor::Authenticated { token } => Some(AuthenticatedUser {
                token: token.clone(),
            }),
        }
    }

    pub fn is_permission_granted_to_all(&self, permission: Permission) -> bool {
        BLANKET_PERMISSION_GRANT.contains(&permission)
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



/// An authenticated user with a valid JWT token.
pub struct AuthenticatedUser {
    token: JWTClaims,
}

impl AuthenticatedUser {
    /// Returns the date and time the user's access token was created,
    /// i.e. when the user logged in.
    #[allow(dead_code)]
    pub fn logged_in_at(&self) -> &DateTime<Utc> {
        &self.token.iat
    }

    /// Returns the date and time the user's access token will expire.
    #[allow(dead_code)]
    pub fn login_expires_at(&self) -> &DateTime<Utc> {
        &self.token.exp
    }

    /// Returns the ID of the user who owns the token.
    pub fn user_id(&self) -> i32 {
        self.token.user_id
    }

    /// Returns a list of permissions this user effectively has.
    /// The permissions are computed by doing a union of all permissions
    /// for each role the user has (since standalone permissions don't exist,
    /// only in combination with roles).
    ///
    /// This operation performs a database lookup.
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
    ///
    /// This operation performs a database lookup.
    pub async fn has_permission<C: ConnectionTrait>(
        &self,
        database: &C,
        permission: Permission,
    ) -> Result<bool> {
        if BLANKET_PERMISSION_GRANT.contains(&permission) {
            return Ok(true);
        }

        UserRoleQuery::user_has_permission(database, self.token.user_id, permission)
            .await
            .wrap_err("Could not query whether the user has a specific permission.")
    }

    /// Returns a list of roles the user has.
    ///
    /// This operation performs a database lookup.
    pub async fn roles<C: ConnectionTrait>(&self, database: &C) -> Result<RoleSet> {
        let role_set = UserRoleQuery::user_roles(database, self.token.user_id)
            .await
            .wrap_err("Could not query roles for user.")?;

        Ok(role_set)
    }
}
