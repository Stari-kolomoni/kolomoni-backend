use actix_web::{web, Scope};

mod endpoints;
pub use endpoints::*;
mod model_impls;


#[rustfmt::skip]
#[allow(clippy::let_and_return)]
pub fn english_dictionary_router() -> Scope {
    let english_dictionary_scope = web::scope("/english");
    let english_dictionary_scope = english_word_router(english_dictionary_scope);
    let english_dictionary_scope = english_word_meaning_router(english_dictionary_scope);
    
    english_dictionary_scope
}
