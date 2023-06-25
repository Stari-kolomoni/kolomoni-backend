use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::{get, HttpRequest, HttpResponse, Responder};
use serde::Serialize;
use tracing::error;

use crate::impl_json_responder_on_serializable;

#[derive(Serialize)]
pub struct PingJSONResponse {
    ok: bool,
}

impl_json_responder_on_serializable!(PingJSONResponse, "PingJSONResponse");


#[get("/ping")]
pub async fn ping() -> impl Responder {
    PingJSONResponse { ok: true }
}
