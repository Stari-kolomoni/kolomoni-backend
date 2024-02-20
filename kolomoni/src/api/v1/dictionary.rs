use actix_web::{web, Scope};

use self::{
    english_word::english_dictionary_router,
    slovene_word::slovene_dictionary_router,
    suggestions::suggested_translations_router,
};

pub mod english_word;
pub mod slovene_word;
pub mod suggestions;


#[rustfmt::skip]
pub fn dictionary_router() -> Scope {
    web::scope("/dictionary")
        .service(slovene_dictionary_router())
        .service(english_dictionary_router())
        .service(suggested_translations_router())
}
