use actix_web::{web, Scope};

// TODO
pub fn users_router() -> Scope {
    web::scope("users")
}
