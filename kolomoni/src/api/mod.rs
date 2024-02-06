use actix_web::{web, Scope};

pub mod errors;
pub mod macros;
pub mod v1;

/// Router for the entire public API.
///
/// Lives under the `/api` path and is made up of `/v1` and its sub-routes.
pub fn api_router() -> Scope {
    web::scope("/api").service(v1::v1_api_router())
}
