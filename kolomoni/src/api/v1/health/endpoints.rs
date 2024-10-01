use actix_web::get;
use kolomoni_core::api_models::PingResponse;

use crate::api::errors::{EndpointResponseBuilder, EndpointResult};


/// Ping the server.
#[utoipa::path(
    get,
    path = "/health/ping",
    tag = "health",
    responses(
        (
            status = 200,
            description = "Server is alive and well.",
            body = inline(PingResponse),
            example = json!({ "ok": true })
        )
    )
)]
#[get("/ping")]
pub async fn ping() -> EndpointResult {
    EndpointResponseBuilder::ok()
        .with_json_body(PingResponse { ok: true })
        .build()
}
