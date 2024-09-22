//! The entire Stari Kolomoni API surface.
//!
//! # Development note
//! We use "" instead of "/" in many places (e.g. `#[get("")`, etc.).
//! This allows the user to request e.g. `GET /api/v1/users` **OR** `GET /api/v1/users/` and
//! get the same (correct) endpoint both times.
//!
//! For more information, see [`NormalizePath`][actix_web::middleware::NormalizePath]
//! (in trim mode).

pub mod dictionary;
pub mod login;
pub mod ping;
pub mod users;

use actix_web::{web, Scope};

use self::{dictionary::dictionary_router, login::login_router, users::users_router};

// TODO refactor the API out of the v1 directory, since we currently have only one version (but keep the HTTP path /v1/ prefix!)

/// Router for the entire V1 API.
/// Lives under the `/api/v1` path.
pub fn v1_api_router() -> Scope {
    web::scope("/v1")
        .service(ping::ping)
        .service(users_router())
        .service(login_router())
        .service(dictionary_router())
}
