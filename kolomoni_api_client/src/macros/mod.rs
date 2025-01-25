pub(crate) mod handlers;



/// Expands to a block of code that handles the response using
/// the specified set of response handlers. If none of the handlers
/// catch the error reason, nothing happens (the [`ErrorReason`] is
/// returned from the macro block).
///
/// # Arguments
/// - [`ServerResponse`], and
/// - an array (`[handler, handler, ...]`) of handler types to use.
///   The provided types must implement the [`ErrorResponseHandler`] trait
///   (see e.g. [`handlers::MissingPermissions`] or [`handlers::InvalidUuidFormat`]).
///
///
/// # What Are Response Handlers
/// Response handlers in the context of this macro are all types that implement
/// the [`ErrorResponseHandler`] trait. The implementors are generally defined in the [`handlers`]
/// module.
///
/// For example, the [`handlers::MissingPermissions`] handler will look at the response's error reason
/// (see [`ErrorReason`]) and early-return [`ClientError::missing_permissions`] from the caller function
/// if the reason is missing permissions (see [`ErrorReason::MissingPermissions`]); otherwise it will do nothing.
#[allow(unused)]
macro_rules! handle_error_reasons_or_ignore {
    ($response_status:expr, $error_reason:expr, [$($handler_type:ty),+]) => {
        {
            use $crate::macros::handlers::ErrorResponseHandler;
            use $crate::macros::handlers::ErrorReasonHandlerContext;
            use $crate::macros::handlers::ErrorReasonHandlerDecision;

            let __context = ErrorReasonHandlerContext {
                response_status_code: $response_status
            };

            let __error_reason = $error_reason;

            // Execute each error handler sequentially - if a given one matches,
            // it will return [`ErrorReasonHandlerDecision::EarlyReturnError`],
            // [`ErrorReasonHandlerDecision::Nothing`] otherwise.
            let __error_reason = $(
                {
                    let handler_decision = <$handler_type as ErrorResponseHandler>::handle_error_reason(
                        __error_reason,
                        &__context
                    );

                    match handler_decision {
                        ErrorReasonHandlerDecision::Nothing { returned_error_reason } => {
                            returned_error_reason
                        },
                        ErrorReasonHandlerDecision::EarlyReturnError { error } => {
                            return Err(error.into())
                        }
                    }
                };
            )+

            __error_reason
        }
    };
}

#[allow(unused)]
pub(crate) use handle_error_reasons_or_ignore;


/// Expands to a block of code that handles the response using
/// the specified set of response handlers. If none of the handlers
/// catch the error reason, a [`ClientError`] is early-returned.
/// Therefore this macro will always early-return an error from the
/// perspective of the caller function, but the precise error variant
/// depends on the actual response.
///
/// # Arguments
/// - [`ServerResponse`], and
/// - an array (`[handler, handler, ...]`) of handler types to use.
///   The provided types must implement the [`ErrorResponseHandler`] trait
///   (see e.g. [`handlers::MissingPermissions`] or [`handlers::InvalidUuidFormat`]).
///
/// If you have already read the response's body, you can also substitute the first
/// parameter for the following: `{ reason: <my_error_reason_variable>, status_code: <my_response_status_code>}`
/// where you should substitute `<my_error_reason_variable>` and `<my_response_status_code>` for the appropriate
/// expressions (of types [`ErrorReason`] and [`StatusCode`], respectively).
///
///
/// # What Are Response Handlers
/// Response handlers in the context of this macro are all types that implement
/// the [`ErrorResponseHandler`] trait. The implementors are generally defined in the [`handlers`]
/// module.
///
/// For example, the [`handlers::MissingPermissions`] handler will look at the response's error reason
/// (see [`ErrorReason`]) and early-return [`ClientError::missing_permissions`] from the caller function
/// if the reason is missing permissions (see [`ErrorReason::MissingPermissions`]); otherwise it will do nothing.
macro_rules! handle_error_reasons_or_catch_unexpected_status {

    ({reason: $error_reason:expr, status_code: $response_status:expr}, [$($handler_type:ty),+]) => {
        {
            use $crate::macros::handlers::ErrorResponseHandler;
            use $crate::macros::handlers::ErrorReasonHandlerContext;
            use $crate::macros::handlers::ErrorReasonHandlerDecision;

            let __error_reason = $error_reason.into();

            let __context = ErrorReasonHandlerContext {
                response_status_code: $response_status
            };

            // Execute each error handler sequentially - if a given one matches,
            // it will return [`ErrorReasonHandlerDecision::EarlyReturnError`],
            // [`ErrorReasonHandlerDecision::Nothing`] otherwise.
            let __error_reason = $(
                {
                    let handler_decision = <$handler_type as ErrorResponseHandler>::handle_error_reason(
                        __error_reason,
                        &__context
                    );

                    match handler_decision {
                        ErrorReasonHandlerDecision::Nothing { returned_error_reason } => {
                            returned_error_reason
                        },
                        ErrorReasonHandlerDecision::EarlyReturnError { error } => {
                            return Err(error.into())
                        }
                    }
                };
            )+

            // If none of the error reason handlers matched, we should return an error
            // indicating that there was an unexpected error.
            return Err(
                $crate::errors::ClientError::unexpected_status_code(__context.response_status_code).into()
            );
        }
    };

    ($response:expr, [$($handler_type:ty),+]) => {
        {
            use $crate::macros::handlers::ErrorResponseHandler;
            use $crate::macros::handlers::ErrorReasonHandlerContext;
            use $crate::macros::handlers::ErrorReasonHandlerDecision;

            let __context = ErrorReasonHandlerContext {
                response_status_code: $response.status()
            };

            let __error_reason = $response.error_reason().await?;

            // Execute each error handler sequentially - if a given one matches,
            // it will return [`ErrorReasonHandlerDecision::EarlyReturnError`],
            // [`ErrorReasonHandlerDecision::Nothing`] otherwise.
            let __error_reason = $(
                {
                    let handler_decision = <$handler_type as ErrorResponseHandler>::handle_error_reason(
                        __error_reason,
                        &__context
                    );

                    match handler_decision {
                        ErrorReasonHandlerDecision::Nothing { returned_error_reason } => {
                            returned_error_reason
                        },
                        ErrorReasonHandlerDecision::EarlyReturnError { error } => {
                            return Err(error.into())
                        }
                    }
                };
            )+

            // If none of the error reason handlers matched, we should return an error
            // indicating that there was an unexpected error.
            return Err(
                $crate::errors::ClientError::unexpected_status_code(__context.response_status_code).into()
            );
        }
    };
}

pub(crate) use handle_error_reasons_or_catch_unexpected_status;


/// Expands to code that early-returns [`ClientError::internal_server_error`].
macro_rules! handle_internal_server_error {
    () => {{
        return Err($crate::errors::ClientError::internal_server_error().into());
    }};
}

pub(crate) use handle_internal_server_error;



/// Expands to code that inspects the response status code
/// and early-returns the appropriate [`ClientError`] variant.
///
/// # Arguments
/// The macro expects only one parameter: a HTTP [`StatusCode`].
///
/// # Cases handled
/// - If the status code is `500 Internal Server Error`, the [`handle_internal_server_error`] macro
///   is invoked (see its documentation for details).
/// - Otherwise, a [`ClientError::unexpected_status_code`] is early-returned.
macro_rules! handle_uncaught_status_code {
    ($response_status_code:expr) => {
        if $response_status_code == reqwest::StatusCode::INTERNAL_SERVER_ERROR {
            $crate::macros::handle_internal_server_error!();
        } else {
            return Err(
                $crate::errors::ClientError::unexpected_status_code($response_status_code).into(),
            );
        }
    };
}

pub(crate) use handle_uncaught_status_code;



/// Expands to an `Err` early-return containing the
/// [`ClientError::unexpected_error_reason`] variant ([`ClientError::UnexpectedResponse`]).
///
/// # Arguments
/// This macro expects two arguments (in the following order):
/// - an [`ErrorReason`] (or a type that implements `Into<`[`ErrorReason`]`>`), and
/// - the HTTP [`StatusCode`] that is associated with the unexpected response.
///
///
/// [`ClientError::unexpected_error_reason`]: crate::errors::ClientError::unexpected_error_reason
/// [`ClientError::UnexpectedResponse`]: crate::errors::ClientError::UnexpectedResponse
macro_rules! handle_unexpected_error_reason {
    ($error_reason:expr, $response_status:expr) => {
        return Err(
            $crate::errors::ClientError::unexpected_error_reason(
                $error_reason.into(),
                $response_status,
            )
            .into(),
        )
    };
}

pub(crate) use handle_unexpected_error_reason;
