//! Provides ways of handling errors in API endpoint functions
//! and ways to have those errors automatically turned into correct
//! HTTP error responses when returned as `Err(error)` from those functions.

use std::borrow::{Borrow, Cow};
use std::fmt::{Display, Formatter};

use actix_http::header::{HeaderName, HeaderValue};
use actix_web::body::{BoxBody, MessageBody};
use actix_web::http::{header, StatusCode};
use actix_web::{HttpResponse, ResponseError};
use chrono::{DateTime, Utc};
use kolomoni_core::api_models::{ErrorReason, InvalidJsonBodyReason, ResponseWithErrorReason};
use kolomoni_core::token::JWTCreationError;
use kolomoni_database::entities::UserQueryError;
use kolomoni_database::QueryError;
use serde::Serialize;
use thiserror::Error;
use tracing::error;

use super::macros::construct_last_modified_header_value;
use crate::authentication::AuthenticatedUserError;




/// The most general endpoint handler error type.
///
/// Almost all of the endpoint handler functions tend (or should)
/// return a `Result` with the [`EndpointError`] error type.
/// The reason for this is that throughout the codebase, we've integrated
/// various functions to return either this error type, or an error type that
/// can be losslessly converted into it.
///
/// For example, [`QueryError`]s, which come from
/// the `kolomoni_database` crate, can be `?`-propagated, because we implement
/// `From<QueryError> for EndpointError`, allowing the actual database calls
/// that happen in endpoints to not worry about error conversion.
///
/// See also: [`EndpointResult`].
#[derive(Debug, Error)]
pub enum EndpointError {
    /*
     * Client errors.
     *
     * Reasons are exposed as a HTTP status code + optionally a JSON body.
     */
    /// The endpoint expected a JSON body, but there was either:
    /// - no JSON body sent with the request, or
    /// - an incorrect `Content-Type` header (expected `application/json`).
    MissingJsonBody,

    /// Invalid JSON body, either due to invalid JSON syntax,
    /// a deserialization error, or because the body is too large.
    InvalidJsonBody { reason: InvalidJsonBodyReason },

    /// Invalid UUID format encountered when trying to convert from a string.
    InvalidUuidFormat {
        #[source]
        error: uuid::Error,
    },

    /*
     * Server errors.
     *
     * Strings and boxed errors are not exposed externally.
     */
    /// Internal error with a reason string.
    ///
    /// Triggers a `500 Internal Server Error`. **The reason doesn't leak through the API**.
    InternalErrorWithReason { reason: Cow<'static, str> },

    /// Internal error, constructed from a boxed [`Error`].
    ///
    /// Triggers a `500 Internal Server Error`. **The reason doesn't leak through the API**.
    InternalGenericError {
        #[from]
        #[source]
        error: Box<dyn std::error::Error>,
    },

    /// Internal error, constructed from a database error ([`sqlx::Error`]).
    ///
    /// Triggers a `500 Internal Server Error`. **The reason doesn't leak through the API**.
    InternalDatabaseError {
        #[from]
        #[source]
        error: sqlx::Error,
    },

    /// Internal database inconsistency, with more details provided by the `problem` field.
    ///
    /// Triggers a `500 Internal Server Error`. **The reason doesn't leak through the API**.
    InvalidDatabaseState { problem: Cow<'static, str> },
}

impl EndpointError {
    /// Creates a client error specifying that a JSON body was expected (but not found).
    ///
    /// This can also happen when the `Content-Type` header isn't set properly.
    pub const fn missing_json_body() -> Self {
        Self::MissingJsonBody
    }

    /// Creates a client error specifying that a JSON body was found, but was invalid.
    ///
    /// Reasons include: invalid JSON syntax, data (schema) error when deserializing,
    /// or body size.
    pub const fn invalid_json_body(reason: InvalidJsonBodyReason) -> Self {
        Self::InvalidJsonBody { reason }
    }

    /// Creates an internal server error based on any [`Error`]-implementing type.
    ///
    /// The error will be boxed internally, but its details will not be exposed
    /// in the API response.
    #[allow(unused)]
    pub fn internal_error<E>(error: E) -> Self
    where
        E: std::error::Error + 'static,
    {
        Self::InternalGenericError {
            error: Box::new(error),
        }
    }

    /// Creates an internal server error based on a [`sqlx::Error`].
    ///
    /// The details of the error will not be exposed in the API response.
    #[allow(unused)]
    pub fn internal_database_error(error: sqlx::Error) -> Self {
        Self::InternalDatabaseError { error }
    }

    /// Creates an internal server error based on a string describing the cause.
    ///
    /// The reason is not exposed in the API response.
    #[inline]
    pub fn internal_error_with_reason<S>(reason: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self::InternalErrorWithReason {
            reason: reason.into(),
        }
    }

    /// Creates an internal server error based on a string describing
    /// how a database state is invalid or inconsistent.
    ///
    /// We tend to use this as sanity checks when performing a sequence
    /// of operation in e.g. a transaction: we fetch some data, verify that it matches,
    /// then e.g. delete the row. If we see that the deletion failed, we would consider
    /// that an invalid database state, because we were in a transaction and the row
    /// previously existed.
    ///
    /// The details of this problem will not be exposed in the API response.
    #[inline]
    pub fn invalid_database_state<S>(problem: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self::InvalidDatabaseState {
            problem: problem.into(),
        }
    }
}

impl Display for EndpointError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingJsonBody => {
                write!(f, "Expected a JSON body.")
            }
            Self::InvalidJsonBody { reason } => match reason {
                InvalidJsonBodyReason::NotJson => {
                    write!(f, "Invalid JSON body: not JSON.")
                }
                InvalidJsonBodyReason::InvalidData => {
                    write!(f, "Invalid JSON body: invalid data.")
                }
                InvalidJsonBodyReason::TooLarge => {
                    write!(f, "Invalid JSON body: too large.")
                }
            },
            Self::InvalidUuidFormat { error } => {
                write!(f, "Invalid UUID format: {}.", error)
            }
            Self::InternalErrorWithReason { reason } => write!(
                f,
                "Internal server error (with reason): {reason}."
            ),
            Self::InternalGenericError { error } => {
                write!(f, "Internal server error (generic): {error:?}")
            }
            Self::InternalDatabaseError { error } => {
                write!(
                    f,
                    "Internal server error (database error): {error}."
                )
            }
            Self::InvalidDatabaseState { problem: reason } => {
                write!(
                    f,
                    "Inconsistent internal database state: {}",
                    reason
                )
            }
        }
    }
}

impl ResponseError for EndpointError {
    /// In reality, because we implemented error_response below,
    /// this function will never be called (status codes from error_response will be used).
    /// (see [`ResponseError::status_code`]).
    fn status_code(&self) -> StatusCode {
        match self {
            Self::MissingJsonBody => StatusCode::BAD_REQUEST,
            Self::InvalidJsonBody { .. } => StatusCode::BAD_GATEWAY,
            Self::InvalidUuidFormat { .. } => StatusCode::BAD_REQUEST,
            Self::InternalErrorWithReason { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InternalGenericError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InternalDatabaseError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InvalidDatabaseState { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        // TODO Find a way to log certain types of these errors (maybe via tracing?)

        let fallibly_built_response = match self {
            Self::MissingJsonBody => EndpointResponseBuilder::bad_request()
                .with_error_reason(ErrorReason::missing_json_body())
                .build(),
            Self::InvalidJsonBody { reason } => EndpointResponseBuilder::bad_request()
                .with_error_reason(ErrorReason::invalid_json_body(*reason))
                .build(),
            Self::InvalidUuidFormat { .. } => EndpointResponseBuilder::bad_request()
                .with_error_reason(ErrorReason::invalid_uuid_format())
                .build(),
            Self::InternalErrorWithReason { .. } => {
                EndpointResponseBuilder::internal_server_error().build()
            }
            Self::InternalGenericError { .. } => {
                EndpointResponseBuilder::internal_server_error().build()
            }
            Self::InternalDatabaseError { .. } => {
                EndpointResponseBuilder::internal_server_error().build()
            }
            Self::InvalidDatabaseState { .. } => {
                EndpointResponseBuilder::internal_server_error().build()
            }
        };


        fallibly_built_response.unwrap_or_else(|_| HttpResponse::InternalServerError().finish())
    }
}


impl From<QueryError> for EndpointError {
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

impl From<UserQueryError> for EndpointError {
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

impl From<AuthenticatedUserError> for EndpointError {
    fn from(value: AuthenticatedUserError) -> Self {
        match value {
            AuthenticatedUserError::QueryError { error } => Self::from(error),
        }
    }
}

impl From<JWTCreationError> for EndpointError {
    fn from(value: JWTCreationError) -> Self {
        match value {
            JWTCreationError::JWTError { error } => Self::InternalGenericError {
                error: Box::new(error),
            },
        }
    }
}




/// An endpoint response building error
/// (returned from the [`EndpointResponseBuilder::build`] method).
#[derive(Debug, Error)]
pub enum EndpointResponseBuilderError {
    #[error("failed to serialize value as JSON")]
    JsonSerializationError {
        #[from]
        #[source]
        error: serde_json::Error,
    },
}



/// A builder for HTTP responses returned from endpoint handlers.
pub struct EndpointResponseBuilder {
    status_code: StatusCode,

    body: Option<Result<Vec<u8>, serde_json::Error>>,

    additional_headers: Vec<(HeaderName, HeaderValue)>,
}

/// Endpoint response builder initialization methods, named after
/// the status codes they initialize with.
impl EndpointResponseBuilder {
    fn new(status_code: StatusCode) -> Self {
        Self {
            status_code,
            body: None,
            additional_headers: Vec::with_capacity(1),
        }
    }

    /// Initializes a response builder with [`StatusCode::OK`].
    #[inline]
    pub fn ok() -> Self {
        Self::new(StatusCode::OK)
    }

    /// Initializes a response builder with [`StatusCode::BAD_REQUEST`].
    #[inline]
    pub fn bad_request() -> Self {
        Self::new(StatusCode::BAD_REQUEST)
    }

    /// Initializes a response builder with [`StatusCode::FORBIDDEN`].
    #[inline]
    pub fn forbidden() -> Self {
        Self::new(StatusCode::FORBIDDEN)
    }

    /// Initializes a response builder with [`StatusCode::UNAUTHORIZED`].
    #[inline]
    pub fn unauthorized() -> Self {
        Self::new(StatusCode::UNAUTHORIZED)
    }

    /// Initializes a response builder with [`StatusCode::CONFLICT`].
    #[inline]
    pub fn conflict() -> Self {
        Self::new(StatusCode::CONFLICT)
    }

    /// Initializes a response builder with [`StatusCode::NOT_FOUND`].
    #[inline]
    pub fn not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND)
    }

    /// Initializes a response builder with [`StatusCode::NOT_MODIFIED`].
    #[inline]
    pub fn not_modified() -> Self {
        Self::new(StatusCode::NOT_MODIFIED)
    }

    /// Initializes a response builder with [`StatusCode::INTERNAL_SERVER_ERROR`].
    #[inline]
    pub fn internal_server_error() -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

/// Endpoint response builder response customization methods.
impl EndpointResponseBuilder {
    /// Sets the JSON body to be included in the response body.
    ///
    /// Potential serialization errors are propagated to the [`build`] method.
    ///
    ///
    /// [`build`]: Self::build
    pub fn with_json_body<D, S>(mut self, data: D) -> Self
    where
        S: Serialize,
        D: Borrow<S>,
    {
        let body = serde_json::to_vec(data.borrow());

        self.additional_headers.push((
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
        ));

        Self {
            status_code: self.status_code,
            body: Some(body),
            additional_headers: self.additional_headers,
        }
    }

    /// Sets the JSON body to be included in the response body
    /// to [`ResponseWithErrorReason`] with the given error reason
    /// (e.g. [`ErrorReason`], [`CategoryErrorReason`], ...).
    ///
    /// Potential serialization errors are propagated to the [`build`] method.
    ///
    ///
    /// [`build`]: Self::build
    pub fn with_error_reason<R>(self, reason: R) -> Self
    where
        R: Into<ErrorReason>,
    {
        self.with_json_body(ResponseWithErrorReason::new(reason.into()))
    }

    /// Sets the `Last-Modified` header to the specified date and time.
    pub fn with_last_modified_at(mut self, last_modified_at: &DateTime<Utc>) -> Self {
        self.additional_headers.push((
            header::LAST_MODIFIED,
            construct_last_modified_header_value(last_modified_at),
        ));

        Self {
            status_code: self.status_code,
            body: self.body,
            additional_headers: self.additional_headers,
        }
    }

    /// Finalizes the builder into a [`HttpResponse`].
    pub fn build(self) -> Result<HttpResponse<BoxBody>, EndpointError> {
        let optional_body = match self.body {
            Some(body_or_error) => match body_or_error {
                Ok(body) => Some(body),
                Err(serialization_error) => {
                    return Err(EndpointError::internal_error(serialization_error))
                }
            },
            None => None,
        };


        let mut response_builder = HttpResponse::build(self.status_code);

        for (header_name, header_value) in self.additional_headers {
            response_builder.insert_header((header_name, header_value));
        }


        match optional_body {
            Some(body) => response_builder
                .message_body(body.boxed())
                // This will, however, never produce an error (`type Error = Infallible`),
                // see <https://docs.rs/actix-web/4.9.0/actix_web/body/trait.MessageBody.html#impl-MessageBody-for-Vec%3Cu8%3E>.
                .map_err(EndpointError::internal_error),
            None => response_builder
                .message_body(().boxed())
                // This will, however, never produce an error (`type Error = Infallible`),
                // see <https://docs.rs/actix-web/4.9.0/actix_web/body/trait.MessageBody.html#impl-MessageBody-for-()>.
                .map_err(EndpointError::internal_error),
        }
    }
}




/// Short for [`Result`]`<`[`HttpResponse`]`, `[`EndpointError`]`>`, intended to be used in most
/// places in handlers of the Stari Kolomoni API.
///
/// The generic parameter (`Body`) specifies which body type is used inside [`HttpResponse`].
/// It defaults to [`BoxBody`], which is what [`EndpointResponseBuilder`] uses
/// and will likely be the most common body type.
///
/// See documentation for [`EndpointError`] for more info.
pub type EndpointResult<Body = BoxBody> = Result<HttpResponse<Body>, EndpointError>;
