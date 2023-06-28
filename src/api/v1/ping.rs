use actix_web::body::BoxBody;
use actix_web::{get, HttpRequest, HttpResponse, Responder};
use serde::Serialize;
use utoipa::ToSchema;

use crate::api::errors::EndpointResult;
use crate::api::macros::DumbResponder;
use crate::impl_json_responder;

#[derive(Serialize, ToSchema)]
pub struct PingResponse {
    ok: bool,
}

impl_json_responder!(PingResponse);


/// Ping the server
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
