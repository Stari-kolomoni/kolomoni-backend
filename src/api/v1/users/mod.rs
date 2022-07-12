mod routes;
mod schemas;
mod models;

use actix_web::{web, Scope};

pub fn users_router() -> Scope {
    web::scope("/users")
    .service(routes::register_user)
    .service(routes::login_user)
    .service(routes::refresh_user_token)
    .service(routes::retrieve_current_user)
    .service(routes::update_current_user)
    .service(routes::delete_current_user)
    .service(routes::retrieve_user)
}