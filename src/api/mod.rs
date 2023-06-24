use actix_web::{web, Scope};

pub mod v1;

pub fn api_router() -> Scope {
    web::scope("/api").service(v1::v1_api_router())
}
