use actix_web::get;
use serde::Serialize;
use utoipa::ToSchema;

use crate::api::errors::EndpointResult;
use crate::api::macros::ContextlessResponder;
use crate::impl_json_response_builder;



#[derive(Serialize, ToSchema)]
pub struct PingResponse {
    ok: bool,
}

impl_json_response_builder!(PingResponse);


/// Ping the server.
#[utoipa::path(
    get,
    path = "/ping",
    tag = "miscellaneous",
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
