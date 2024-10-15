use std::borrow::Cow;

use kolomoni_core::{api_models::ErrorReason, permissions::Permission};
use reqwest::StatusCode;
use thiserror::Error;



#[derive(Debug, Error)]
pub enum ClientError {
    #[error("failed to prepare a URL")]
    UrlPreparationError {
        #[from]
        #[source]
        error: url::ParseError,
    },

    #[error("failed to serialize body data as JSON")]
    RequestBodySerializationError {
        #[source]
        error: serde_json::Error,
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

    #[error(
        "server refused the request due to missing caller permissions: {:?}",
        .permissions
    )]
    MissingPermissions { permissions: Vec<Permission> },

    #[error("server returned a 500 Internal Server Error")]
    InternalServerError,

    /// Indicates that th
    #[error(
        "server sent an unexpected and unhandled {} response \
        (may indicate this client being out of date with the API): {}",
        .status_code,
        .reason
    )]
    UnexpectedResponse {
        status_code: StatusCode,
        reason: Cow<'static, str>,
    },
}

impl ClientError {
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


pub type ClientResult<V, E = ClientError> = Result<V, E>;
