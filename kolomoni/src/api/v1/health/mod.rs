mod endpoints;
use actix_web::web;
pub use endpoints::*;


pub fn health_router() -> actix_web::Scope {
    web::scope("/health").service(ping)
}
