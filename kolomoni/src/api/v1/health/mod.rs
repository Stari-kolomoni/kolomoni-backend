mod endpoints;
use actix_web::web;
pub use endpoints::*;

mod model_impls;


pub fn health_router() -> actix_web::Scope {
    web::scope("/health").service(ping)
}
