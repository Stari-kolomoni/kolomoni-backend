use actix_web::body::BoxBody;
use actix_web::{get, HttpRequest, HttpResponse, Responder};
use serde::Serialize;

use crate::api::errors::EndpointResult;
use crate::api::macros::DumbResponder;
use crate::impl_json_responder;

#[derive(Serialize)]
pub struct PingJSONResponse {
    ok: bool,
}

impl_json_responder!(PingJSONResponse);


#[get("/ping")]
pub async fn ping() -> EndpointResult {
    Ok(PingJSONResponse { ok: true }.into_response())
}
