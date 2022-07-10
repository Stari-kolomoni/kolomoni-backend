use actix_web::{web, Scope};

mod users;

pub fn v1_router() -> Scope {
    web::scope("/v1")
        .service(users::users_router())
}