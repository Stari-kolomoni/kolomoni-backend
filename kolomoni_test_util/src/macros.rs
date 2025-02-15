#[macro_export]
macro_rules! ensure_request_is_ok {
    ($result_with_client_error:expr, $error_message:expr) => {{
        $result_with_client_error.unwrap_or_else(|error| panic!("{}\n{:?}", $error_message, error))
    }};

    ($result_with_client_error:expr) => {{
        $result_with_client_error
            .unwrap_or_else(|error| panic!("response is not Ok\n{:?}", $error_message, error))
    }};
}

pub use assert_matches::*;
pub use ensure_request_is_ok;
