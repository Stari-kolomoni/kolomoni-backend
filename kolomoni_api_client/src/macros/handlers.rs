use kolomoni_core::api_models::ErrorReason;
use reqwest::StatusCode;

use crate::errors::ClientError;

pub enum ErrorReasonHandlerDecision<E> {
    Nothing {
        /// Gives back ownership of the error reason instance
        /// to the caller.
        returned_error_reason: ErrorReason,
    },

    EarlyReturnError {
        error: E,
    },
}

impl<E> ErrorReasonHandlerDecision<E> {
    #[inline]
    pub const fn nothing(returned_error_reason: ErrorReason) -> Self {
        Self::Nothing {
            returned_error_reason,
        }
    }

    #[inline]
    pub const fn early_return_error(error: E) -> Self {
        Self::EarlyReturnError { error }
    }
}


#[derive(Clone)]
pub struct ErrorReasonHandlerContext {
    pub response_status_code: StatusCode,
}

pub trait ErrorResponseHandler {
    type Error;

    fn handle_error_reason(
        error_reason: ErrorReason,
        context: &ErrorReasonHandlerContext,
    ) -> ErrorReasonHandlerDecision<Self::Error>;
}




pub struct MissingPermissions;

impl ErrorResponseHandler for MissingPermissions {
    type Error = ClientError;

    fn handle_error_reason(
        error_reason: ErrorReason,
        context: &ErrorReasonHandlerContext,
    ) -> ErrorReasonHandlerDecision<Self::Error> {
        if let ErrorReason::MissingPermissions { .. } = &error_reason {
            return ErrorReasonHandlerDecision::early_return_error(
                ClientError::unexpected_error_reason(error_reason, context.response_status_code),
            );
        }

        ErrorReasonHandlerDecision::nothing(error_reason)
    }
}


pub struct InvalidUuidFormat;

impl ErrorResponseHandler for InvalidUuidFormat {
    type Error = ClientError;

    fn handle_error_reason(
        error_reason: ErrorReason,
        context: &ErrorReasonHandlerContext,
    ) -> ErrorReasonHandlerDecision<Self::Error> {
        if error_reason == ErrorReason::InvalidUuidFormat {
            return ErrorReasonHandlerDecision::early_return_error(
                ClientError::unexpected_response(
                    context.response_status_code,
                    "server (unexpectedly) could not parse the provided UUID, \
                    even though we provided a valid UUID",
                ),
            );
        }

        ErrorReasonHandlerDecision::nothing(error_reason)
    }
}
