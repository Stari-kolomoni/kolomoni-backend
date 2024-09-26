use actix_web::{web, Scope};


mod endpoints;
pub use endpoints::*;
mod model_impls;


// TODO Word meaning links.


#[rustfmt::skip]
pub fn slovene_dictionary_router() -> Scope {
    web::scope("/slovene")
        .service(slovene_word_router())
        .service(slovene_word_meaning_router())
}
