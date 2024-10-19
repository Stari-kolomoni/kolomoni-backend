pub(crate) mod handlers;


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

pub(crate) use handle_error_reasons_or_ignore;


macro_rules! handle_error_reasons_or_catch_unexpected_status {
    ($response:expr, [$($handler_type:ty),+]) => {
        {
            use $crate::macros::handlers::ErrorResponseHandler;
            use $crate::macros::handlers::ErrorReasonHandlerContext;
            use $crate::macros::handlers::ErrorReasonHandlerDecision;

            let __context = ErrorReasonHandlerContext {
                response_status_code: $response.status()
            };

            let __error_reason = $response.json_error_reason().await?;

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



macro_rules! internal_server_error {
    () => {{
        return Err($crate::errors::ClientError::internal_server_error().into());
    }};
}

pub(crate) use internal_server_error;


macro_rules! unexpected_status_code {
    ($response_status_code:expr) => {{
        return Err(
            $crate::errors::ClientError::unexpected_status_code($response_status_code).into(),
        );
    }};
}

pub(crate) use unexpected_status_code;



macro_rules! unexpected_error_reason {
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

pub(crate) use unexpected_error_reason;
