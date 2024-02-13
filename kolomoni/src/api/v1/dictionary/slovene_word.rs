//! TODO We need:
//! - GET /dictionary/slovene - lists all slovene words
//! - POST /dictionary/slovene - creates a slovene word
//! - GET /dictionary/slovene/{uuid} - gets a slovene word by uuid
//! - PATCH /dictionary/slovene/{uuid} - updates a slovene word
//! - DELETE /dictionary/slovene/{uuid} - deltes a slovene word
//! Leave links alone for now. Leave suggestions and translations alone for now.
//!

use actix_web::{get, post, web, Scope};
use chrono::{DateTime, Utc};
use kolomoni_database::{entities, query};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    api::{
        errors::{APIError, EndpointResult},
        macros::ContextlessResponder,
    },
    authentication::UserAuthenticationExtractor,
    impl_json_response_builder,
    state::ApplicationState,
};

#[derive(Serialize, Debug, ToSchema)]
pub struct SloveneWord {
    pub word_id: String,
    pub lemma: String,
    pub disambiguation: Option<String>,
    pub description: Option<String>,
    pub added_at: DateTime<Utc>,
    pub last_edited_at: DateTime<Utc>,
}

impl SloveneWord {
    pub fn from_database_model(model: entities::word_slovene::Model) -> Self {
        Self {
            word_id: model.word_id.to_string(),
            lemma: model.lemma,
            disambiguation: model.disambiguation,
            description: model.description,
            added_at: model.added_at.to_utc(),
            last_edited_at: model.last_edited_at.to_utc(),
        }
    }
}



#[derive(Serialize, Debug, ToSchema)]
pub struct AllSloveneWordsResponse {
    slovene_words: Vec<SloveneWord>,
}

impl_json_response_builder!(AllSloveneWordsResponse);


#[get("")]
pub async fn get_all_slovene_words(state: ApplicationState) -> EndpointResult {
    // Load words from the database.
    let words = query::SloveneWordQuery::all_words(&state.database)
        .await
        .map_err(APIError::InternalError)?;

    let words_as_api_structures = words
        .into_iter()
        .map(SloveneWord::from_database_model)
        .collect();

    Ok(AllSloveneWordsResponse {
        slovene_words: words_as_api_structures,
    }
    .into_response())
}


#[derive(Deserialize, Clone, Debug, ToSchema)]
pub struct SloveneWordCreationRequest {
    pub lemma: String,
    pub disambiguation: Option<String>,
    pub description: Option<String>,
}


#[post("")]
pub async fn create_a_slovene_word(
    _state: ApplicationState,
    _user_auth: UserAuthenticationExtractor,
    _creation_request: web::Json<SloveneWordCreationRequest>,
) -> EndpointResult {
    todo!();
}


#[rustfmt::skip]
pub fn slovene_dictionary_router() -> Scope {
    web::scope("/slovene")
        .service(get_all_slovene_words)
        .service(create_a_slovene_word)
}
