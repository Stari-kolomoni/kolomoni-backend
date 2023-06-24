pub mod users;

use actix_web::{web, Scope};

pub fn v1_api_router() -> Scope {
    web::scope("/v1").service(users::users_router())
}