use std::collections::HashSet;

use actix_utils::future;
use actix_utils::future::Ready;
use actix_web::dev::Payload;
use actix_web::http::{header, StatusCode};
use actix_web::web::Data;
use actix_web::{error, FromRequest, HttpRequest};
use anyhow::{anyhow, Context, Result};
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use crate::database::queries;
use crate::jwt::{JWTClaims, JWTValidationError};
use crate::state::AppState;

// User permissions that we have (inspired by the scope system in OAuth).
// The defined permissions must match with the `*_seed_permissions.rs` file in `migrations`!
#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Copy, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum UserPermission {
    /// Allows the user to log in and view their account information.
    #[serde(rename = "user.self:read")]
    UserSelfRead,

    /// Allows the user to update their account information.
    #[serde(rename = "user.self:write")]
    UserSelfWrite,

    /// Allows the user to view public account information of any other user.
    #[serde(rename = "user.any:read")]
    UserRead,

    /// Allows the user to update public account information of any other user.
    #[serde(rename = "user.any:write")]
    UserWrite,
}

impl UserPermission {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "user.self:read" => Some(Self::UserSelfRead),
            "user.self:write" => Some(Self::UserSelfWrite),
            "user.any:read" => Some(Self::UserRead),
            "user.any:write" => Some(Self::UserWrite),
            _ => None,
        }
    }

    pub fn to_name(self) -> &'static str {
        match self {
            UserPermission::UserSelfRead => "user.self:read",
            UserPermission::UserSelfWrite => "user.self:write",
            UserPermission::UserRead => "user.any:read",
            UserPermission::UserWrite => "user.any:write",
        }
    }

    pub fn to_id(self) -> i32 {
        match self {
            UserPermission::UserSelfRead => 1,
            UserPermission::UserSelfWrite => 2,
            UserPermission::UserRead => 3,
            UserPermission::UserWrite => 4,
        }
    }
}

// List of user permissions given to newly-registered users.
pub const DEFAULT_USER_PERMISSIONS: [UserPermission; 3] = [
    UserPermission::UserSelfRead,
    UserPermission::UserSelfWrite,
    UserPermission::UserRead,
];


pub struct UserPermissions {
    permissions: HashSet<UserPermission>,
}

impl UserPermissions {
    pub async fn get_from_database(database: &DbConn, username: &str) -> Result<Self> {
        let permission_names =
            queries::user_permissions::Query::get_user_permission_names_by_username(
                database, username,
            )
            .await
            .with_context(|| "Failed to get user permissions from database.")?
            .ok_or_else(|| anyhow!("Failed to get user permissions: no such user."))?;

        let permissions = permission_names
            .into_iter()
            .map(|permission_name| {
                UserPermission::from_name(&permission_name)
                    .ok_or_else(|| anyhow!("BUG: No such permission: {permission_name}!"))
            })
            .collect::<Result<HashSet<UserPermission>>>()?;

        Ok(Self { permissions })
    }

    pub fn has_permission(&self, permission: UserPermission) -> bool {
        self.permissions.contains(&permission)
    }
}


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

    pub async fn permissions(&self, database: &DbConn) -> Result<Option<UserPermissions>> {
        match self {
            UserAuth::Unauthenticated => Ok(None),
            UserAuth::Authenticated { token } => {
                let user_permissions =
                    UserPermissions::get_from_database(database, &token.username).await?;

                Ok(Some(user_permissions))
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
