use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::{get, HttpRequest, HttpResponse, Responder};
use serde::Serialize;

#[derive(Serialize)]
pub struct PingJSONResponse {
    ok: bool,
}

impl Responder for PingJSONResponse {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let body_encoded_json = serde_json::to_string(&self);

        match body_encoded_json {
            Ok(body_encoded_json) => HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(body_encoded_json),
            Err(_) => HttpResponse::InternalServerError().finish(),
        }
    }
}

#[get("/ping")]
pub async fn ping() -> impl Responder {
    PingJSONResponse { ok: true }
}
