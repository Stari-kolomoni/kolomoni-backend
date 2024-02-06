use std::fmt::{Display, Formatter};

use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use kolomoni_auth::permissions::Permission;
use sea_orm::DbErr;
use serde::Serialize;
use thiserror::Error;
use tracing::error;
use utoipa::ToSchema;

/// Simple JSON-encodable response containing a single field: a `reason`.
/// Useful for specifying reasons when returning a HTTP status code with an error.
#[derive(Serialize, Debug, ToSchema)]
pub struct ErrorReasonResponse {
    /// Error reason.
    pub reason: String,
}

impl ErrorReasonResponse {
    /// Initialize an [`ErrorReasonResponse`] with a custom error message.
    pub fn custom_reason<M: Into<String>>(reason: M) -> Self {
        Self {
            reason: reason.into(),
        }
    }

    /// Initialize an [`ErrorReasonResponse`] with a message about a missing `Authorization` header.
    pub fn not_authenticated() -> Self {
        Self {
            reason: "Not authenticated (missing Authorization header).".to_string(),
        }
    }

    /// Initialize an [`ErrorReasonResponse`] with a message about missing permissions
    /// (but not specifying which ones).
    pub fn missing_permissions() -> Self {
        Self {
            reason: "Missing permissions.".to_string(),
        }
    }

    /// Initialize an `ErrorReasonResponse` with a message about a specific missing permission.
    pub fn missing_specific_permission(permission: Permission) -> Self {
        Self {
            reason: format!("Missing permission: {}", permission.name()),
        }
    }
}


/// General Stari Kolomoni API error type.
/// Use alongside [`EndpointResult`] in your actix endpoint handlers to easily [`?`][std::try]-return errors.
///
///
/// # Examples
///
/// ## Internal errors
/// If the function you're calling returns an [`miette::Result`], you can simply
/// map it to [`APIError::InternalError`] and use `?` to return early if an error occurred:
///
/// ```
/// # use miette::miette;
/// # use serde::Serialize;
/// # use actix_web::get;
/// # use kolomoni::impl_json_responder;
/// # use kolomoni::api::errors::APIError;
/// # use kolomoni::api::macros::DumbResponder;
/// # use kolomoni::api::errors::EndpointResult;
/// #[derive(Serialize)]
/// struct RandomValueResponse {
///     value: i32,
/// }
///
/// impl_json_responder!(RandomValueResponse);
///
///
/// #[get("/some/path")]
/// async fn example_internal_error() -> EndpointResult {
///     let some_value: i32 = Result::Err(miette!("This is some error."))
///         .map_err(APIError::InternalError)?;
///     
///     println!("{}", some_value);
///     Ok(RandomValueResponse { value: some_value }.into_response())
/// }
/// ```
///
/// > Note that in this case `.map_err(APIError::InternalError)` is the same as
/// > the longer version: `.map_err(|error| APIError::InternalError(error))`.
///
/// Similarly, you can map a [`sea_orm::error::DbErr`] to [`APIError::InternalDatabaseError`].
/// If you are working with some other type of `Result`, you can do something like this instead
/// to produce an `500 Internal Server Error` on `Err`:
///
/// ```
/// # use miette::Error;
/// # use kolomoni::api::errors::{APIError, EndpointResult};
/// async fn example_string_internal_error() -> EndpointResult {
///     let some_value: i32 = Result::<i32, Error>::Ok(42)
///         .map_err(|_| APIError::internal_reason("Failed this operation!"))?;
///
///     # todo!();
/// }
/// ```
///
///
/// ### Other (not found / not authenticated / missing permissions)
/// Just like [`APIError::internal_reason`], which returns a constructed [`APIError::InternalReason`]
/// with your message, there are other helper methods, such as [`APIError::not_found`],
/// [`APIError::not_found_with_reason`], [`APIError::not_enough_permissions`]
/// and [`APIError::missing_specific_permission`].
///
///
/// ### Full authentication example
/// When the user is not authenticated at all, you can use the [`APIError::NotAuthenticated`] variant.
///
/// What follows is a full authentication and permission example, requiring the user to
/// be authenticated and have the `user.self:read` permission:
///
/// ```
/// # use actix_web::{post, web};
/// # use kolomoni::require_permission;
/// # use kolomoni::authentication::UserAuth;
/// # use kolomoni::state::AppState;
/// # use kolomoni_auth::permissions::UserPermission;
/// # use kolomoni::api::errors::{APIError, EndpointResult};
/// # use kolomoni::api::macros::DumbResponder;
/// #[post("/some/path")]
/// async fn example_auth(
///     user_auth: UserAuth,
///     state: web::Data<AppState>,
/// ) -> EndpointResult {
///     let (token, permissions) = user_auth
///         .token_and_permissions_if_authenticated(&state.database)
///         .await
///         .map_err(APIError::InternalError)?
///         .ok_or_else(|| APIError::NotAuthenticated)?;
///
///     require_permission!(permissions, UserPermission::UserSelfRead);
///
///     # todo!();
/// }
/// ```
#[derive(Debug, Error)]
pub enum APIError {
    /// User is not authenticated (missing `Authorization` header).
    NotAuthenticated,

    /// User does not have the required permissions.
    /// If `Some`, this specifies the missing permission.
    NotEnoughPermissions {
        missing_permission: Option<Permission>,
    },

    /// Resource could not be found. If `Some`, this describes the reason for a 404.
    NotFound {
        reason_response: Option<ErrorReasonResponse>,
    },

    /// Internal error with a string reason.
    /// Triggers a `500 Internal Server Error` (*doesn't leak the error through the API*).
    InternalReason(String),

    /// Internal error, constructed from an [`miette::Error`].
    /// Triggers a `500 Internal Server Error` (*doesn't leak the error through the API*).
    InternalError(miette::Error),

    /// Internal error, constructed from an [`sea_orm::error::DbErr`].
    /// Triggers a `500 Internal Server Error` (*doesn't leak the error through the API*).
    InternalDatabaseError(DbErr),
}

impl APIError {
    /// Initialize a new not found API error without a specific reason.
    #[inline]
    pub fn not_found() -> Self {
        Self::NotFound {
            reason_response: None,
        }
    }

    /// Initialize a new not found API error with a specific reason.
    #[allow(dead_code)]
    #[inline]
    pub fn not_found_with_reason<M: Into<String>>(reason: M) -> Self {
        Self::NotFound {
            reason_response: Some(ErrorReasonResponse::custom_reason(reason)),
        }
    }

    /// Initialize a new not enough permissions API error without a specific permission.
    #[allow(dead_code)]
    #[inline]
    pub fn not_enough_permissions() -> Self {
        Self::NotEnoughPermissions {
            missing_permission: None,
        }
    }

    /// Initialize a new not enough permissions API error with a specific permission.
    #[inline]
    pub fn missing_specific_permission(permission: Permission) -> Self {
        Self::NotEnoughPermissions {
            missing_permission: Some(permission),
        }
    }

    /// Initialize a new internal API error with an internal reason (not leaked through the API).
    #[inline]
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
                    missing_permission.name()
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

/// Short for [`Result`]`<`[`HttpResponse`]`, `[`APIError`]`>`, intended to be used in most
/// places in handlers of the Stari Kolomoni API.
///
/// See documentation for [`APIError`] for more info.
pub type EndpointResult = Result<HttpResponse, APIError>;
