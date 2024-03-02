use std::str::FromStr;

use actix_web::{web, Scope};
use kolomoni_database::entities;
use miette::IntoDiagnostic;
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use self::{
    categories::categories_router,
    english_word::english_dictionary_router,
    search::search_router,
    slovene_word::slovene_dictionary_router,
    suggestions::suggested_translations_router,
    translations::translations_router,
};
use crate::api::errors::APIError;

pub mod categories;
pub mod english_word;
pub mod search;
pub mod slovene_word;
pub mod suggestions;
pub mod translations;


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct Category {
    pub id: i32,
    pub slovene_name: String,
    pub english_name: String,
}

impl Category {
    pub fn from_database_model(model: entities::category::Model) -> Self {
        Self {
            id: model.id,
            slovene_name: model.slovene_name,
            english_name: model.english_name,
        }
    }
}



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
        .service(categories_router())
        .service(search_router())
}
