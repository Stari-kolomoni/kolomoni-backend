//! Provides ways of handling errors in API endpoint functions
//! and ways to have those errors automatically turned into correct
//! HTTP error responses when returned as `Err(error)` from those functions.

use std::borrow::Cow;
use std::fmt::{Display, Formatter};

use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use itertools::Itertools;
use kolomoni_auth::{JWTCreationError, Permission};
use kolomoni_database::entities::UserQueryError;
use kolomoni_database::QueryError;
use serde::Serialize;
use thiserror::Error;
use tracing::error;
use utoipa::ToSchema;

use super::macros::{KolomoniResponseBuilderJSONError, KolomoniResponseBuilderLMAError};
use crate::authentication::AuthenticatedUserError;


/// Simple JSON-encodable response containing a single field: a `reason`.
///
/// This is useful for specifying reasons when returning a HTTP status code
/// with an error.
#[derive(Serialize, PartialEq, Eq, Clone, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(serde::Deserialize))]
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
    pub fn missing_specific_permission(permission: &Permission) -> Self {
        Self {
            reason: format!("Missing permission: {}", permission.name()),
        }
    }

    /// Initialize an `ErrorReasonResponse` with a message about a specific missing permission
    /// or permissions.
    pub fn missing_specific_permissions(permission: &[Permission]) -> Self {
        if permission.is_empty() {
            Self::missing_permissions()
        } else if permission.len() == 1 {
            Self::missing_specific_permission(
                // PANIC SAFETY: We just checked that the length is one.
                permission.first().unwrap(),
            )
        } else {
            Self {
                reason: format!(
                    "Missing permissions: {}",
                    permission
                        .iter()
                        .map(|permission| permission.name())
                        .join(", ")
                ),
            }
        }
    }
}



/// General-purpose Stari Kolomoni API error type.
///
/// Use this type alongside an [`EndpointResult`] return type in your actix endpoint handlers
/// to allow you to easily
/// [`?`](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#a-shortcut-for-propagating-errors-the--operator)-return
/// errors and automatically convert them into HTTP 4xx and 5xx errors!
/// For more details on how this works, consult the
/// [Actix documentation on errors](https://actix.rs/docs/errors) and the
/// `impl `[`ResponseError`]` for `[`APIError`] block.
///
/// <br>
///
/// # Usage examples
///
/// ## 1.1 Internal errors
/// If the function you're calling returns a [`miette::Result`], you can simply
/// map it to an [`APIError::InternalError`] and use `?` to return early if an error occurred.
/// If it returns a std-compatible [`Error`][std::error::Error], you must first call
/// [`error.into_diagnostic()`][miette::IntoDiagnostic::into_diagnostic].
///
/// ```
/// # use miette::miette;
/// # use serde::Serialize;
/// # use actix_web::get;
/// # use kolomoni::impl_json_response_builder;
/// # use kolomoni::api::errors::APIError;
/// # use kolomoni::api::macros::ContextlessResponder;
/// # use kolomoni::api::errors::EndpointResult;
/// #[derive(Serialize)]
/// struct RandomValueResponse {
///     value: i32,
/// }
///
/// impl_json_response_builder!(RandomValueResponse);
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
/// As mentioned, returning an [`APIError`] from a [`EndpointResult`]-returning actix handler
/// will mean actix will automatically generate a relevant 4xx/5xx error, including any additional info,
/// as configured.
///
/// ---
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
/// ## 1.2 Other errors (not found, missing permissions, etc.)
/// Just like [`APIError::internal_reason`], which returns a constructed [`APIError::InternalReason`]
/// with your message, there are other helper methods, such as:
/// - [`APIError::not_found`],
/// - [`APIError::not_found_with_reason`],
/// - [`APIError::missing_permission`],
/// - [`APIError::missing_specific_permission`], and
/// - [`APIError::missing_specific_permissions`].
///
///
/// <br>
///
/// # Full authentication example
/// When the user is not authenticated at all, you can use the
/// [`APIError::NotAuthenticated`] error variant.
///
/// What follows is a full authentication and permission example, requiring the user to
/// be authenticated and have the `user.self:read` permission.
///
/// ```
/// use actix_web::post;
/// use kolomoni::{require_permission, require_authentication};
/// use kolomoni::authentication::UserAuthenticationExtractor;
/// use kolomoni::state::ApplicationState;
/// use kolomoni_auth::Permission;
/// use kolomoni::api::errors::{APIError, EndpointResult};
///
/// #[post("/some/path")]
/// async fn example_auth(
///     state: ApplicationState,
///     authentication: UserAuthenticationExtractor,
/// ) -> EndpointResult {
///     let authenticated_user = require_authentication!(authentication);
///     require_permission!(state, authenticated_user, Permission::UserSelfRead);
///
///     // ... the rest of the function ...
///     # todo!();
/// }
/// ```
#[derive(Debug, Error)]
pub enum APIError {
    /// User is not authenticated
    /// (missing `Authorization: Bearer <token>` HTTP header).
    NotAuthenticated,

    /// User does not have the required permission (or permissions).
    /// If `Some`, this specifies the missing permission (or permissions).
    NotEnoughPermissions {
        missing_permission: Option<Vec<Permission>>,
    },

    /// Resource could not be found. If `Some`, this describes the reason for a 404.
    NotFound {
        reason_response: Option<ErrorReasonResponse>,
    },

    /// Bad client request with a reason; will produce a `400 Bad Request`.
    /// The `reason` will also be sent along in the response.
    OtherClientError { reason: Cow<'static, str> },

    /// Internal error with a string reason.
    /// Triggers a `500 Internal Server Error` (**reason doesn't leak through the API**).
    InternalErrorWithReason { reason: Cow<'static, str> },

    /// Internal error, constructed from a boxed [`Error`].
    /// Triggers a `500 Internal Server Error` (**error doesn't leak through the API**).
    InternalGenericError {
        #[from]
        #[source]
        error: Box<dyn std::error::Error>,
    },

    /// Internal error, constructed from a [`sqlx::Error`].
    /// Triggers a `500 Internal Server Error` (*doesn't leak the error through the API*).
    InternalDatabaseError {
        #[from]
        #[source]
        error: sqlx::Error,
    },
}

impl APIError {
    /// Initialize a new not found API error without a specific reason.
    #[inline]
    pub const fn not_found() -> Self {
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

    /// Initialize a new API error, clarifying that the user is missing
    /// a permission (or multiple permissions), but without clarification as to which those are.
    #[allow(dead_code)]
    #[inline]
    pub const fn missing_permission() -> Self {
        Self::NotEnoughPermissions {
            missing_permission: None,
        }
    }

    pub fn client_error<S>(reason: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self::OtherClientError {
            reason: Cow::from(reason.into()),
        }
    }

    /// Initialize a new API error, clarifying that the user is missing
    /// some permission.
    #[inline]
    pub fn missing_specific_permission(permission: Permission) -> Self {
        Self::NotEnoughPermissions {
            missing_permission: Some(vec![permission]),
        }
    }

    /// Initialize a new API error, clarifying that the user is missing
    /// some set of permissions.
    #[inline]
    #[allow(dead_code)]
    pub const fn missing_specific_permissions(permissions: Vec<Permission>) -> Self {
        Self::NotEnoughPermissions {
            missing_permission: Some(permissions),
        }
    }

    pub fn internal_error<E>(error: E) -> Self
    where
        E: std::error::Error + 'static,
    {
        Self::InternalGenericError {
            error: Box::new(error),
        }
    }

    pub fn internal_database_error(error: sqlx::Error) -> Self {
        Self::InternalDatabaseError { error }
    }

    /// Initialize a new internal API error using an internal reason string.
    /// When constructing an HTTP response using this error variant, the **reason
    /// is not leaked through the API.**
    #[inline]
    pub fn internal_error_with_reason<S>(reason: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self::InternalErrorWithReason {
            reason: reason.into(),
        }
    }
}

impl Display for APIError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            APIError::NotAuthenticated => write!(f, "No authentication."),
            APIError::NotEnoughPermissions { missing_permission } => match missing_permission {
                Some(missing_permission) => {
                    if missing_permission.len() == 1 {
                        write!(
                            f,
                            "User doesn't have the required permission: {}",
                            // PANIC SAFETY: We just checked that length is 1.
                            missing_permission.first().unwrap().name()
                        )
                    } else {
                        write!(
                            f,
                            "User doesn't have the required permissions: {}",
                            missing_permission
                                .iter()
                                .map(|permission| permission.name())
                                .join(", ")
                        )
                    }
                }
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
            APIError::OtherClientError { reason } => write!(f, "Client error: {}", reason),
            APIError::InternalErrorWithReason { reason } => write!(
                f,
                "Internal server error (with reason): {reason}."
            ),
            APIError::InternalGenericError { error } => {
                write!(f, "Internal server error (generic): {error:?}")
            }
            APIError::InternalDatabaseError { error } => {
                write!(
                    f,
                    "Internal server error (database error): {error}."
                )
            }
        }
    }
}

impl ResponseError for APIError {
    fn status_code(&self) -> StatusCode {
        match self {
            APIError::NotAuthenticated => StatusCode::UNAUTHORIZED,
            APIError::NotEnoughPermissions { .. } => StatusCode::FORBIDDEN,
            APIError::NotFound { .. } => StatusCode::NOT_FOUND,
            APIError::OtherClientError { .. } => StatusCode::BAD_REQUEST,
            APIError::InternalErrorWithReason { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            APIError::InternalGenericError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            APIError::InternalDatabaseError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self {
            APIError::NotAuthenticated => {
                HttpResponse::Unauthorized().json(ErrorReasonResponse::not_authenticated())
            }
            APIError::NotEnoughPermissions { missing_permission } => match missing_permission {
                Some(missing_permissions) => HttpResponse::Forbidden().json(
                    ErrorReasonResponse::missing_specific_permissions(missing_permissions),
                ),
                None => HttpResponse::Forbidden().json(ErrorReasonResponse::missing_permissions()),
            },
            APIError::NotFound { reason_response } => match reason_response {
                Some(reason_response) => HttpResponse::NotFound().json(reason_response),
                None => HttpResponse::NotFound().finish(),
            },
            APIError::OtherClientError { reason } => {
                HttpResponse::BadRequest().json(ErrorReasonResponse {
                    reason: reason.to_string(),
                })
            }
            APIError::InternalErrorWithReason { reason } => {
                error!(error = %reason, "Internal database error (custom reason).");

                HttpResponse::InternalServerError().finish()
            }
            APIError::InternalGenericError { error } => {
                error!(
                    error = ?error,
                    "Internal server error (generic)."
                );

                HttpResponse::InternalServerError().finish()
            }
            APIError::InternalDatabaseError { error } => {
                error!(
                    error = ?error,
                    "Internal server error (database error).",
                );

                HttpResponse::InternalServerError().finish()
            }
        }
    }
}


impl From<QueryError> for APIError {
    fn from(value: QueryError) -> Self {
        match value {
            QueryError::SqlxError { error } => Self::InternalDatabaseError { error },
            QueryError::ModelError { reason } => Self::InternalErrorWithReason { reason },
            QueryError::DatabaseInconsistencyError { problem: reason } => {
                Self::InternalErrorWithReason { reason }
            }
        }
    }
}

impl From<UserQueryError> for APIError {
    fn from(value: UserQueryError) -> Self {
        match value {
            UserQueryError::SqlxError { error } => Self::InternalDatabaseError { error },
            UserQueryError::ModelError { reason } => Self::InternalErrorWithReason { reason },
            UserQueryError::HasherError { error } => Self::InternalGenericError {
                error: Box::new(error),
            },
            UserQueryError::DatabaseConsistencyError { reason } => {
                Self::InternalErrorWithReason { reason }
            }
        }
    }
}

impl From<AuthenticatedUserError> for APIError {
    fn from(value: AuthenticatedUserError) -> Self {
        match value {
            AuthenticatedUserError::QueryError { error } => Self::from(error),
        }
    }
}

impl From<JWTCreationError> for APIError {
    fn from(value: JWTCreationError) -> Self {
        match value {
            JWTCreationError::JWTError { error } => Self::InternalGenericError {
                error: Box::new(error),
            },
        }
    }
}

impl From<KolomoniResponseBuilderJSONError> for APIError {
    fn from(value: KolomoniResponseBuilderJSONError) -> Self {
        match value {
            KolomoniResponseBuilderJSONError::JsonError { error } => Self::InternalGenericError {
                error: Box::new(error),
            },
        }
    }
}

impl From<KolomoniResponseBuilderLMAError> for APIError {
    fn from(value: KolomoniResponseBuilderLMAError) -> Self {
        match value {
            KolomoniResponseBuilderLMAError::JsonError { error } => Self::InternalGenericError {
                error: Box::new(error),
            },
        }
    }
}




/// Short for [`Result`]`<`[`HttpResponse`]`, `[`APIError`]`>`, intended to be used in most
/// places in handlers of the Stari Kolomoni API.
///
/// The generic parameter (`Body`) specifies which body type is used inside [`HttpResponse`]
/// and defaults to [`BoxBody`], which is what
/// [`KolomoniResponseBuilder`][super::macros::KolomoniResponseBuilder]
/// uses and will likely be the most common body type.
///
/// See documentation for [`APIError`] for more info.
pub type EndpointResult<Body = BoxBody> = Result<HttpResponse<Body>, APIError>;
