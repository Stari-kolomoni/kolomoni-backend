use actix_web::{Scope, web};

mod v1;

pub fn api_router() -> Scope {
    web::scope("/api")
        .service(v1::v1_router())
}