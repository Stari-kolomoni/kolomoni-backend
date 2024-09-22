use actix_web::{web, Scope};
use meaning::slovene_word_meaning_router;
use word::slovene_word_router;



pub mod meaning;
pub mod word;



// TODO Links.


#[rustfmt::skip]
pub fn slovene_dictionary_router() -> Scope {
    web::scope("/slovene")
        .service(slovene_word_router())
        .service(slovene_word_meaning_router())
}
