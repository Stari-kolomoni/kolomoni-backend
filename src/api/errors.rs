use std::fmt::{Display, Formatter};

use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use sea_orm::DbErr;
use serde::Serialize;
use thiserror::Error;
use tracing::error;
use utoipa::ToSchema;

use crate::api::auth::UserPermission;

#[derive(Serialize, Debug, ToSchema)]
pub struct ErrorReasonResponse {
    /// Error reason.
    pub reason: String,
}

impl ErrorReasonResponse {
    pub fn custom_reason<M: Into<String>>(reason: M) -> Self {
        Self {
            reason: reason.into(),
        }
    }

    pub fn not_authenticated() -> Self {
        Self {
            reason: "Not authenticated (missing Authorization header).".to_string(),
        }
    }

    pub fn missing_permissions() -> Self {
        Self {
            reason: "Missing permissions.".to_string(),
        }
    }

    pub fn missing_specific_permission(permission: UserPermission) -> Self {
        Self {
            reason: format!("Missing permission: {}", permission.to_name()),
        }
    }
}



#[derive(Debug, Error)]
pub enum APIError {
    NotAuthenticated,

    NotEnoughPermissions {
        missing_permission: Option<UserPermission>,
    },

    NotFound {
        reason_response: Option<ErrorReasonResponse>,
    },

    InternalReason(String),

    InternalError(anyhow::Error),

    InternalDatabaseError(DbErr),
}

impl APIError {
    pub fn not_found() -> Self {
        Self::NotFound {
            reason_response: None,
        }
    }

    #[allow(dead_code)]
    pub fn not_found_with_reason<M: Into<String>>(reason: M) -> Self {
        Self::NotFound {
            reason_response: Some(ErrorReasonResponse::custom_reason(reason)),
        }
    }

    #[allow(dead_code)]
    pub fn not_enough_permissions() -> Self {
        Self::NotEnoughPermissions {
            missing_permission: None,
        }
    }

    pub fn missing_specific_permission(permission: UserPermission) -> Self {
        Self::NotEnoughPermissions {
            missing_permission: Some(permission),
        }
    }

    pub fn internal_reason(reason: &str) -> Self {
        Self::InternalReason(reason.to_string())
    }
}

impl Display for APIError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            APIError::NotAuthenticated => write!(f, "No authentication."),
            APIError::NotEnoughPermissions { missing_permission } => match missing_permission {
                Some(missing_permission) => write!(
                    f,
                    "User doesn't have the required permission: {}",
                    missing_permission.to_name()
                ),
                None => write!(f, "User doesn't have enough permissions."),
            },
            APIError::NotFound { reason_response } => match reason_response {
                Some(reason) => {
                    write!(f, "Resource not found: {}", reason.reason)
                }
                None => {
                    write!(f, "Resource not found.")
                }
            },
            APIError::InternalReason(reason) => write!(f, "Internal error: {reason}."),
            APIError::InternalError(error) => write!(f, "Internal error: {error}."),
            APIError::InternalDatabaseError(error) => write!(f, "Internal database error: {error}."),
        }
    }
}

impl ResponseError for APIError {
    fn status_code(&self) -> StatusCode {
        match self {
            APIError::NotAuthenticated => StatusCode::UNAUTHORIZED,
            APIError::NotEnoughPermissions { .. } => StatusCode::FORBIDDEN,
            APIError::NotFound { .. } => StatusCode::NOT_FOUND,
            APIError::InternalReason(_) => StatusCode::INTERNAL_SERVER_ERROR,
            APIError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            APIError::InternalDatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self {
            APIError::NotAuthenticated => {
                HttpResponse::Unauthorized().json(ErrorReasonResponse::not_authenticated())
            }
            APIError::NotEnoughPermissions { missing_permission } => match missing_permission {
                Some(missing_permission) => HttpResponse::Forbidden().json(
                    ErrorReasonResponse::missing_specific_permission(*missing_permission),
                ),
                None => HttpResponse::Forbidden().json(ErrorReasonResponse::missing_permissions()),
            },
            APIError::NotFound { reason_response } => match reason_response {
                Some(reason_response) => HttpResponse::NotFound().json(reason_response),
                None => HttpResponse::NotFound().finish(),
            },
            APIError::InternalReason(error) => {
                error!(error = error, "Internal error.");

                HttpResponse::InternalServerError().finish()
            }
            APIError::InternalError(error) => {
                error!(
                    error = error.to_string(),
                    "Internal server error."
                );

                HttpResponse::InternalServerError().finish()
            }
            APIError::InternalDatabaseError(error) => {
                error!(
                    error = error.to_string(),
                    "Internal database error.",
                );

                HttpResponse::InternalServerError().finish()
            }
        }
    }
}

/// Short for `Result<HttpResponse, APIError>`, intended to be used in most
/// places in handlers of the Stari Kolomoni API.
pub type EndpointResult = Result<HttpResponse, APIError>;
