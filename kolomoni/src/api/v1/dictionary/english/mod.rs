use actix_web::{web, Scope};
use meaning::english_word_meaning_router;
use word::english_word_router;

pub mod meaning;
pub mod word;

// TODO Links.


#[rustfmt::skip]
pub fn english_dictionary_router() -> Scope {
    web::scope("/english")
        .service(english_word_router())
        .service(english_word_meaning_router())
}
