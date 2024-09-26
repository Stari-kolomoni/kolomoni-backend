use actix_web::{web, Scope};

mod endpoints;
pub use endpoints::*;
mod model_impls;


// TODO Word meaning links.


#[rustfmt::skip]
pub fn english_dictionary_router() -> Scope {
    web::scope("/english")
        .service(english_word_router())
        .service(english_word_meaning_router())
}
