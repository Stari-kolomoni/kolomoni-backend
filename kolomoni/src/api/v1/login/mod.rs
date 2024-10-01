mod endpoints;

use actix_web::{web, Scope};
pub use endpoints::*;



#[rustfmt::skip]
pub fn login_router() -> Scope {
    web::scope("/login")
        .service(login)
        .service(refresh_login)
}
