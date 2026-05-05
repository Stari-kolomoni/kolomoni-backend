use kolomoni_core::api_models::ErrorReason;
use reqwest::StatusCode;

use crate::{
    client::errors::RequestError,
    response::{raw::RawResponse, ResponseValueError},
};


/// If the provided `error_reason` indicates one or more missing permissions,
/// this function returns [`Err(E::from_request_error)`] with a [`RequestError::missing_permissions`].
///
/// Otherwise, it returns [`Ok`].
///
/// This function returns a [`Result<(), E>`], because we want to be able to ergonomically
/// `?`-return the error case, but ignore the unit return value otherwise (see endpoints
/// for examples). However, this will require manually providing the error type `E`
/// using the turbofish syntax (at least in Rust 1.94), but I think the tradeoff is mostly worth it.
#[inline(always)]
#[must_use = "the potential error must be handled"]
pub(crate) fn err_if_missing_permissions<E>(
    _status: StatusCode,
    error_reason: &ErrorReason,
) -> Result<(), E>
where
    E: ResponseValueError,
{
    if let ErrorReason::MissingPermissions { permissions } = error_reason {
        return Err(E::from_request_error(
            RequestError::missing_permissions(permissions.to_owned()),
        ));
    }

    Ok(())
}

/// If the provided `error_reason` indicates an invalid UUID present in the request,
/// this function returns [`Err(E::from_request_error)`] with a [`RequestError::unexpected_response`].
///
/// Otherwise, it returns [`Ok`].
///
/// This function returns a [`Result<(), E>`], because we want to be able to ergonomically
/// `?`-return the error case, but ignore the unit return value otherwise (see endpoints
/// for examples). However, this will require manually providing the error type `E`
/// using the turbofish syntax (at least in Rust 1.94), but I think the tradeoff is mostly worth it.
#[inline(always)]
#[must_use = "the potential error must be handled"]
pub(crate) fn err_if_invalid_uuid<E>(status: StatusCode, error_reason: &ErrorReason) -> Result<(), E>
where
    E: ResponseValueError,
{
    if error_reason == &ErrorReason::InvalidUuidFormat {
        return Err(E::from_request_error(
            RequestError::unexpected_response(
                status,
                "server (unexpectedly) could not parse the provided UUID, \
                even though we provided (we think) a valid UUID",
            ),
        ));
    }

    Ok(())
}



/// Returns [`Err(E::from_request_error)`]
/// that indicates either an internal server error
/// (if status is HTTP 500 Internal Server Error),
/// or an error that indicates we didn't expect to see this status code.
///
/// Prefer using [`unexpected_response`] instead, if possible.
///
/// # Cases handled
/// - If the status code is `500 Internal Server Error`, a [`RequestError::internal_server_error`]
/// - Otherwise, a general [`RequestError::unexpected_status_code`] is constructed.
#[inline(always)]
#[allow(dead_code)]
pub(crate) fn unexpected_status_code<E>(status: StatusCode) -> Result<(), E>
where
    E: ResponseValueError,
{
    if status == reqwest::StatusCode::INTERNAL_SERVER_ERROR {
        Err(E::from_request_error(
            RequestError::internal_server_error(),
        ))
    } else {
        Err(E::from_request_error(
            RequestError::unexpected_status_code(status),
        ))
    }
}

/// Returns [`E::from_request_error`] with a request error
/// that indicates either an internal server error (if the status
/// is a 500 Internal Server Error), or a request error
/// that indicates we didn't expect to see this status code.
///
/// # Cases handled
/// - If the status code is `500 Internal Server Error`, a [`RequestError::internal_server_error`]
/// - Otherwise, we try to deserialize the response as an [`ErrorReason`].
///   - If we succeed, [`RequestError::unexpected_error_reason`] is returned,
///   - otherwise, a general [`RequestError::unexpected_response`] is constructed.
#[inline(always)]
pub(crate) async fn unexpected_response<E>(response: RawResponse) -> E
where
    E: ResponseValueError,
{
    let status = response.status();

    if status == StatusCode::INTERNAL_SERVER_ERROR {
        E::from_request_error(RequestError::internal_server_error())
    } else {
        let error_response_result = response.error_reason().await;
        match error_response_result {
            Ok(error_reason) => E::from_request_error(RequestError::unexpected_error_reason(
                error_reason,
                status,
            )),
            Err(_) => {
                // This might just mean that the server did not respond with an ErrorReason,
                // which is certainly possible.
                E::from_request_error(RequestError::unexpected_response(
                    status,
                    "unexpected response (no error reason provided)",
                ))
            }
        }
    }
}

/// Returns [`E::from_request_error`] with a request error
/// that indicates we did not expect to see this error reason.
///
/// See also: [`RequestError::unexpected_error_reason`].
#[inline(always)]
pub(crate) fn unexpected_error_reason<E, R>(status: StatusCode, error_reason: R) -> E
where
    R: Into<ErrorReason>,
    E: ResponseValueError,
{
    E::from_request_error(RequestError::unexpected_error_reason(
        error_reason.into(),
        status,
    ))
}
