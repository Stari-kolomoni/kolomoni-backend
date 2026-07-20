use kolomoni_test_core::prelude::*;

#[tokio::test]
async fn server_can_be_pinged() {
    let client = fresh_test_client!();

    client.assertable_health().assert_server_ping_is_ok().await;
}
