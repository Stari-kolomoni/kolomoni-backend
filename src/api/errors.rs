use std::fmt::{Display, Formatter};

use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use sea_orm::DbErr;
use serde::Serialize;
use thiserror::Error;
use tracing::error;

#[derive(Serialize, Debug)]
pub struct ErrorReasonResponse {
    reason: String,
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

    pub fn not_enough_permissions() -> Self {
        Self {
            reason: "Missing permissions.".to_string(),
        }
    }
}



#[derive(Debug, Error)]
pub enum APIError {
    NotAuthenticated,

    NotEnoughPermissions,

    InternalReason(String),

    InternalError(anyhow::Error),

    InternalDatabaseError(DbErr),
}

impl APIError {
    pub fn internal_reason(reason: &str) -> Self {
        Self::InternalReason(reason.to_string())
    }
}

impl Display for APIError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            APIError::NotAuthenticated => write!(f, "No authentication."),
            APIError::NotEnoughPermissions => write!(f, "User doesn't have enough permissions."),
            APIError::InternalReason(reason) => write!(f, "Internal error: {reason}."),
            APIError::InternalError(error) => write!(f, "Internal error: {error}."),
            APIError::InternalDatabaseError(error) => write!(f, "Internal database error: {error}."),
        }
    }
}

impl ResponseError for APIError {
    fn status_code(&self) -> StatusCode {
        match self {
            APIError::NotAuthenticated => StatusCode::FORBIDDEN,
            APIError::NotEnoughPermissions => StatusCode::FORBIDDEN,
            APIError::InternalReason(_) => StatusCode::INTERNAL_SERVER_ERROR,
            APIError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            APIError::InternalDatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self {
            APIError::NotAuthenticated => {
                HttpResponse::Forbidden().json(ErrorReasonResponse::not_authenticated())
            }
            APIError::NotEnoughPermissions => {
                HttpResponse::Forbidden().json(ErrorReasonResponse::not_enough_permissions())
            }
            APIError::InternalReason(error) => {
                error!(error = error, "Internal error.");

                HttpResponse::InternalServerError().finish()
            }
            APIError::InternalError(error) => {
                error!(error = error.to_string(), "Internal error.");

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
