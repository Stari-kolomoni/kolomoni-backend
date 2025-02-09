#[macro_export]
macro_rules! assert_request_ok {
    ($result_with_client_error:expr, $error_message:expr) => {{
        $result_with_client_error.unwrap_or_else(|error| panic!("{}\n{:?}", $error_message, error))
    }};
}

pub use assert_request_ok;
