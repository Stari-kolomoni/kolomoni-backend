use actix_web::{web, Scope};


mod endpoints;
pub use endpoints::*;
mod model_impls;


#[rustfmt::skip]
#[allow(clippy::let_and_return)]
pub fn slovene_dictionary_router() -> Scope {
    let slovene_dictionary_scope = web::scope("/slovene");
    let slovene_dictionary_scope = slovene_word_router(slovene_dictionary_scope);
    let slovene_dictionary_scope = slovene_word_meaning_router(slovene_dictionary_scope);

    slovene_dictionary_scope
}
