use std::str::FromStr;

use actix_web::{web, Scope};
use miette::IntoDiagnostic;
use sea_orm::prelude::Uuid;

use self::{
    english_word::english_dictionary_router,
    slovene_word::slovene_dictionary_router,
    suggestions::suggested_translations_router,
    translations::translations_router,
};
use crate::api::errors::APIError;

pub mod english_word;
pub mod slovene_word;
pub mod suggestions;
pub mod translations;


pub fn parse_string_into_uuid(potential_uuid: &str) -> Result<Uuid, APIError> {
    let target_word_uuid = Uuid::from_str(potential_uuid)
        .into_diagnostic()
        .map_err(|_| APIError::client_error("invalid UUID"))?;

    Ok(target_word_uuid)
}


#[rustfmt::skip]
pub fn dictionary_router() -> Scope {
    web::scope("/dictionary")
        .service(slovene_dictionary_router())
        .service(english_dictionary_router())
        .service(suggested_translations_router())
        .service(translations_router())
}
