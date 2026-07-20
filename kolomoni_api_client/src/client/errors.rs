use std::borrow::Cow;

use kolomoni_core::{api_models::ErrorReason, permissions::Permission};
use reqwest::{
    header::{HeaderName, InvalidHeaderValue},
    StatusCode,
};
use thiserror::Error;


/// Errors that can occur while initializinga Stari Kolomoni client.
///
/// See also: [`UnauthenticatedKolomoniClient::new_with_options`].
#[derive(Debug, Error)]
pub enum ClientInitializationError {
    #[error("unable to initialize reqwest HTTP client")]
    UnableToInitializeHttpClient {
        #[from]
        #[source]
        error: reqwest::Error,
    },
}

/// Client errors that can occur while preparing a request.
///
/// Can be converted into an equivalent [`RequestError`]
/// (which also includes execution errors) using
/// [`Self::into_full_request_error`].
#[derive(Debug, Error)]
pub enum RequestPreparationError {
    /// Failed to prepare the provided URL for the request.
    ///
    /// Reasons may include:
    /// - missing host,
    /// - invalid address or port,
    /// - invalid characters,
    /// - etc.
    #[error("failed to prepare a URL")]
    UrlPreparation {
        #[from]
        #[source]
        error: url::ParseError,
    },

    /// Failed to serialize the provided data into JSON.
    ///
    /// This could for example be due to the data
    /// containing non-string keys (see [`serde_json::to_vec`]).
    #[error("failed to serialize body data as JSON")]
    RequestBodySerialization {
        #[source]
        error: serde_json::Error,
    },

    #[error("failed to encode a value in the request header")]
    RequestHeaderValueEncoding {
        header_name: HeaderName,

        #[source]
        error: InvalidHeaderValue,
    },

    /// Failed to build the underlying HTTP request instance ([`reqwest::Error`]).
    #[error("failed to prepare the request")]
    RequestInstancePreparation {
        #[from]
        #[source]
        error: reqwest::Error,
    },
}

impl RequestPreparationError {
    /// Losslessly converts `self` into [`RequestError`]
    /// (because [`Self`] is just a subset of [`RequestError`]).
    pub fn into_full_request_error(self) -> RequestError {
        match self {
            Self::UrlPreparation { error } => RequestError::UrlPreparationError { error },
            Self::RequestBodySerialization { error } => {
                RequestError::RequestBodySerializationError { error }
            }
            Self::RequestHeaderValueEncoding { header_name, error } => {
                RequestError::RequestHeaderValueEncodingError { header_name, error }
            }
            Self::RequestInstancePreparation { error } => {
                RequestError::RequestPreparationError { error }
            }
        }
    }
}

/// Client errors that can occur while preparing a request,
/// executing a request, or parsing its response.
#[derive(Debug, Error)]
pub enum RequestError {
    /// Failed to prepare the provided URL for the request.
    ///
    /// Reasons may include:
    /// - missing host,
    /// - invalid address or port,
    /// - invalid characters,
    /// - etc.
    #[error("failed to prepare a URL")]
    UrlPreparationError {
        #[from]
        #[source]
        error: url::ParseError,
    },

    /// Failed to serialize the provided data into JSON.
    ///
    /// This could for example be due to the data
    /// containing non-string keys (see [`serde_json::to_vec`]).
    #[error("failed to serialize body data as JSON")]
    RequestBodySerializationError {
        #[source]
        error: serde_json::Error,
    },

    #[error("failed to encode a value in the request header")]
    RequestHeaderValueEncodingError {
        header_name: HeaderName,

        #[source]
        error: InvalidHeaderValue,
    },

    /// Failed to build the underlying HTTP request instance ([`reqwest::Error`]).
    #[error("failed to prepare the request")]
    RequestPreparationError {
        #[from]
        #[source]
        error: reqwest::Error,
    },

    #[error("failed while executing HTTP request")]
    RequestExecutionError {
        #[source]
        error: reqwest::Error,
    },

    #[error(
        "failed to extract JSON body from response \
        (either invalid JSON syntax or mismatching content schema)"
    )]
    ResponseJsonBodyError {
        #[source]
        error: serde_json::Error,
    },

    #[error("server refused the request due to missing authentication")]
    MissingAuthentication,

    #[error(
        "server refused the request due to missing caller permissions: {:?}",
        .permissions
    )]
    MissingPermissions { permissions: Vec<Permission> },

    #[error("server returned a 500 Internal Server Error")]
    InternalServerError,

    #[error(
        "Server sent an unexpected, unhandled, or malformed response (status code: {}). \
        This may indicate this client being out of date with the API. {}",
        .status_code,
        .reason
    )]
    UnexpectedResponse {
        status_code: StatusCode,
        reason: Cow<'static, str>,
    },
}

impl RequestError {
    #[inline]
    pub(crate) const fn internal_server_error() -> Self {
        Self::InternalServerError
    }

    #[inline]
    pub(crate) const fn missing_permissions(permissions: Vec<Permission>) -> Self {
        Self::MissingPermissions { permissions }
    }

    #[inline]
    pub(crate) fn unexpected_error_reason(
        unexpected_error_reason: ErrorReason,
        status_code: StatusCode,
    ) -> Self {
        Self::UnexpectedResponse {
            status_code,
            reason: Cow::Owned(format!(
                "unexpected ErrorReason in the response: {:?}",
                unexpected_error_reason
            )),
        }
    }

    #[inline]
    pub(crate) const fn unexpected_status_code(status_code: StatusCode) -> Self {
        Self::UnexpectedResponse {
            status_code,
            reason: Cow::Borrowed("unexpected response status code"),
        }
    }

    #[inline]
    pub(crate) fn unexpected_response<R>(status_code: StatusCode, reason: R) -> Self
    where
        R: Into<Cow<'static, str>>,
    {
        Self::UnexpectedResponse {
            status_code,
            reason: reason.into(),
        }
    }
}


pub(crate) type RequestResult<V, E = RequestError> = Result<V, E>;
