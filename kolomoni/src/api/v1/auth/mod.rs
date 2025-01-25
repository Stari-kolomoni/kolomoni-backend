pub mod endpoints;

use actix_web::{web, Scope};
use endpoints::{login, refresh_login, register_user};

pub fn auth_router() -> Scope {
    web::scope("/auth")
        .service(login)
        .service(refresh_login)
        .service(register_user)
}
