use actix_web::{web, Scope};

use self::slovene_word::slovene_dictionary_router;

mod slovene_word;

#[rustfmt::skip]
pub fn dictionary_router() -> Scope {
    // TODO
    web::scope("/dictionary")
        .service(slovene_dictionary_router())
}
