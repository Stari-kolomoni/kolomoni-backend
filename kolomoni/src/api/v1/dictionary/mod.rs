use std::str::FromStr;

use actix_web::{web, Scope};
use english::english_dictionary_router;
use kolomoni_core::api_models::Category;
use sqlx::types::Uuid;

use self::{
    categories::categories_router,
    // suggestions::suggested_translations_router,
    translations::translations_router,
};
use crate::api::errors::APIError;

pub mod categories;
pub mod english;
// TODO
pub mod slovene;
// TODO
// pub mod search;
// DEPRECATED
// pub mod suggestions;
pub mod translations;




pub fn parse_string_into_uuid(potential_uuid: &str) -> Result<Uuid, APIError> {
    let target_word_uuid =
        Uuid::from_str(potential_uuid).map_err(|_| APIError::client_error("invalid UUID"))?;

    Ok(target_word_uuid)
}


#[rustfmt::skip]
pub fn dictionary_router() -> Scope {
    web::scope("/dictionary")
        // TODO
        // .service(slovene_dictionary_router())
        .service(english_dictionary_router())
        // DEPRECATED
        // .service(suggested_translations_router())
        .service(translations_router())
        .service(categories_router())
        // TODO
        // .service(search_router())
}
