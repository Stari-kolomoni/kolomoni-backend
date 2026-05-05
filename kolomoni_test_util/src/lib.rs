pub mod macros;
pub mod prelude;
pub mod response;
pub mod sample_categories;
pub mod sample_users;
pub mod sample_words;
pub mod server;

use std::env;

use kolomoni_api_client::server::KolomoniApiServer;
use prelude::UnauthanticatedTestServerClient;


const TEST_SERVER_BASE_URL_ENVIRONMENT_VAR_NAME: &str = "KOLOMONI_TEST_SERVER_URL";


pub async fn initialize_fresh_test_server() -> UnauthanticatedTestServerClient {
    let test_server_base_url =
        env::var(TEST_SERVER_BASE_URL_ENVIRONMENT_VAR_NAME).unwrap_or_else(|error| {
            panic!(
                "failed to initialize fresh test server: environment variable {} error: {:?}",
                TEST_SERVER_BASE_URL_ENVIRONMENT_VAR_NAME, error
            )
        });

    let test_server =
        KolomoniApiServer::new_from_server_url_and_base_api_path(&test_server_base_url, "/api/v1/")
            .unwrap_or_else(|error| {
                panic!(
                    "failed to initialize fresh test server: invalid base url: {}",
                    error
                )
            });

    let client = UnauthanticatedTestServerClient::new(test_server);

    client.testing().assert_testing_is_enabled_on_server().await;
    client.testing().perform_full_reset().await;

    client
}
