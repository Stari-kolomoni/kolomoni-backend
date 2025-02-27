use kolomoni_test_util::prelude::*;

#[tokio::test]
async fn server_can_be_pinged() {
    let client = initialize_fresh_test_server().await;

    client
        .assertable_health()
        .assert_server_can_be_pinged()
        .await;
}
