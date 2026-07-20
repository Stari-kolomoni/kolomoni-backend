pub use assert_matches::*;
pub use kolomoni_test_macros::test;

/// Initializes a new [`UnauthanticatedTestClient`], verifies
/// that the test server in question has truly been compiled with test flags,
/// and resets the entire server to a fresh state.
///
/// [`UnauthanticatedTestClient`]: kolomoni_test_core::client::UnauthanticatedTestClient
#[macro_export]
macro_rules! fresh_test_client {
    () => {
        ::kolomoni_test_core::client::UnauthanticatedTestClient::new_with_fresh_server().await
    };
}

pub use fresh_test_client;
