use actix_web::get;
use kolomoni_core::api_models::PingResponse;

use crate::api::errors::EndpointResult;
use crate::api::macros::ContextlessResponder;


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
    Ok(PingResponse { ok: true }.into_response())
}
