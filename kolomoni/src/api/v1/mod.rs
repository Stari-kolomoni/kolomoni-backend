//! # Development note
//!
//! We use "" instead of "/" in many places (e.g. `#[get("")`, etc.)
//! because this allows the user to request e.g. `GET /api/v1/users` OR `GET /api/v1/users/` and
//! get the correct endpoint both times.
//!
//! For more information, see `actix_web::middleware::NormalizePath` (trim mode).

pub mod dictionary;
pub mod login;
pub mod ping;
pub mod users;

use actix_web::{web, Scope};

use self::{dictionary::dictionary_router, login::login_router, users::users_router};


/// Router for the entire V1 API.
/// Lives under the `/api/v1` path.
pub fn v1_api_router() -> Scope {
    web::scope("/v1")
        .service(ping::ping)
        .service(users_router())
        .service(login_router())
        .service(dictionary_router())
}
