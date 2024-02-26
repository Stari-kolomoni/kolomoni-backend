use kolomoni::api::v1::ping::PingResponse;
use kolomoni_test_util::prelude::*;

#[tokio::test]
async fn server_can_be_pinged() {
    let server = initialize_test_server().await;

    let response = server.request(Method::GET, "/api/v1/ping").send().await;

    response.assert_status_equals(StatusCode::OK);
    response.assert_json_body_matches(PingResponse { ok: true });
}
