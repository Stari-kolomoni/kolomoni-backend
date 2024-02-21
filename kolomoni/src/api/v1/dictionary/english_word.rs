use actix_http::StatusCode;
use actix_web::{delete, get, patch, post, web, HttpResponse, Scope};
use chrono::{DateTime, Utc};
use kolomoni_auth::Permission;
use kolomoni_database::{
    entities,
    mutation::{EnglishWordMutation, NewEnglishWord, UpdatedEnglishWord},
    query::{self, EnglishWordQuery, TranslationQuery, TranslationSuggestionQuery},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::slovene_word::SloveneWord;
use crate::{
    api::{
        errors::{APIError, EndpointResult},
        macros::ContextlessResponder,
        openapi,
        v1::dictionary::parse_string_into_uuid,
    },
    authentication::UserAuthenticationExtractor,
    error_response_with_reason,
    impl_json_response_builder,
    require_authentication,
    require_permission,
    require_permission_with_optional_authentication,
    state::ApplicationState,
};



#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
#[schema(
    example = json!({
        "word_id": "018dbe00-266e-7398-abd2-0906df0aa345",
        "lemma": "adventurer",
        "disambiguation": "character",
        "description": "Playable or non-playable character.",
        "added_at": "2023-06-27T20:34:27.217273Z",
        "last_edited_at": "2023-06-27T20:34:27.217273Z"
    })
)]
pub struct EnglishWord {
    /// Internal UUID of the word.
    pub word_id: String,

    /// An abstract or base form of the word.
    pub lemma: String,

    /// If there are multiple similar words, the disambiguation
    /// helps distinguish the word from other words at a glance.
    pub disambiguation: Option<String>,

    /// A short description of the word. Supports Markdown.
    ///
    /// TODO Will need special Markdown support for linking to other dictionary words
    ///      and possibly autocomplete in the frontend editor.
    pub description: Option<String>,

    /// When the word was created.
    pub added_at: DateTime<Utc>,

    /// When the word was last edited.
    pub last_edited_at: DateTime<Utc>,

    /// Suggested slovene translations of this word.
    pub suggested_translations: Vec<SloveneWord>,

    /// Slovene translations of this word.
    pub translations: Vec<SloveneWord>,
}

impl EnglishWord {
    pub fn from_models(
        english_model: entities::word_english::Model,
        suggested_translations: Vec<entities::word_slovene::Model>,
        translations: Vec<entities::word_slovene::Model>,
    ) -> Self {
        let suggested_translations = suggested_translations
            .into_iter()
            .map(SloveneWord::from_database_model)
            .collect();

        let translations = translations
            .into_iter()
            .map(SloveneWord::from_database_model)
            .collect();

        Self {
            word_id: english_model.word_id.to_string(),
            lemma: english_model.lemma,
            disambiguation: english_model.disambiguation,
            description: english_model.description,
            added_at: english_model.added_at.to_utc(),
            last_edited_at: english_model.last_edited_at.to_utc(),
            suggested_translations,
            translations,
        }
    }
}


#[derive(Serialize, Debug, ToSchema)]
pub struct EnglishWordsResponse {
    english_words: Vec<EnglishWord>,
}

impl_json_response_builder!(EnglishWordsResponse);


#[utoipa::path(
    get,
    path = "/dictionary/english",
    tag = "dictionary:english",
    responses(
        (
            status = 200,
            description = "A list of all english words.",
            body = EnglishWordsResponse,
        ),
        openapi::FailedAuthenticationResponses<openapi::RequiresWordRead>,
        openapi::InternalServerErrorResponse,
    )
)]
#[get("")]
pub async fn get_all_english_words(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
) -> EndpointResult {
    require_permission_with_optional_authentication!(state, authentication, Permission::WordRead);

    let words = query::EnglishWordQuery::all_words(&state.database)
        .await
        .map_err(APIError::InternalError)?;


    let mut words_as_api_structures: Vec<EnglishWord> = Vec::with_capacity(words.len());

    // PERF: This might be a good candidate for optimization, probably with caching.
    for raw_english_word in words {
        let suggested_translations =
            query::TranslationSuggestionQuery::suggestions_for_english_word(
                &state.database,
                raw_english_word.word_id,
            )
            .await
            .map_err(APIError::InternalError)?;

        let translations = query::TranslationQuery::translations_for_english_word(
            &state.database,
            raw_english_word.word_id,
        )
        .await
        .map_err(APIError::InternalError)?;


        words_as_api_structures.push(EnglishWord::from_models(
            raw_english_word,
            suggested_translations,
            translations,
        ));
    }


    Ok(EnglishWordsResponse {
        english_words: words_as_api_structures,
    }
    .into_response())
}



#[derive(Deserialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Serialize))]
#[schema(
    example = json!({
        "lemma": "adventurer",
        "disambiguation": "character",
        "description": "Playable or non-playable character.",
    })
)]
pub struct EnglishWordCreationRequest {
    pub lemma: String,
    pub disambiguation: Option<String>,
    pub description: Option<String>,
}


#[derive(Serialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Deserialize))]
#[schema(
    example = json!({
        "word": {
            "word_id": "018dbe00-266e-7398-abd2-0906df0aa345",
            "lemma": "adventurer",
            "disambiguation": "character",
            "description": "Playable or non-playable character.",
            "added_at": "2023-06-27T20:34:27.217273Z",
            "last_edited_at": "2023-06-27T20:34:27.217273Z"
        }
    })
)]
pub struct EnglishWordCreationResponse {
    pub word: EnglishWord,
}

impl_json_response_builder!(EnglishWordCreationResponse);


#[utoipa::path(
    post,
    path = "/dictionary/english",
    tag = "dictionary:english",
    request_body(
        content = EnglishWordCreationRequest
    ),
    responses(
        (
            status = 200,
            description = "The newly-created english word.",
            body = EnglishWordCreationResponse,
        ),
        (
            status = 409,
            description = "English word with the given lemma already exists.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "An english word with the given lemma already exists." })
        ),
        openapi::FailedAuthenticationResponses<openapi::RequiresWordCreate>,
        openapi::InternalServerErrorResponse,
    )
)]
#[post("")]
pub async fn create_english_word(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    creation_request: web::Json<EnglishWordCreationRequest>,
) -> EndpointResult {
    let authenticated_user = require_authentication!(authentication);
    require_permission!(state, authenticated_user, Permission::WordCreate);


    let creation_request = creation_request.into_inner();


    let lemma_already_exists =
        EnglishWordQuery::word_exists_by_lemma(&state.database, creation_request.lemma.clone())
            .await
            .map_err(APIError::InternalError)?;

    if lemma_already_exists {
        return Ok(error_response_with_reason!(
            StatusCode::CONFLICT,
            "An english word with the given lemma already exists."
        ));
    }


    let newly_created_word = EnglishWordMutation::create(
        &state.database,
        NewEnglishWord {
            lemma: creation_request.lemma,
            disambiguation: creation_request.disambiguation,
            description: creation_request.description,
        },
    )
    .await
    .map_err(APIError::InternalError)?;


    Ok(EnglishWordCreationResponse {
        // A newly-created word can not have any suggestions or translations yet.
        word: EnglishWord::from_models(newly_created_word, Vec::new(), Vec::new()),
    }
    .into_response())
}




#[derive(Serialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Deserialize))]
pub struct EnglishWordInfoResponse {
    pub word: EnglishWord,
}

impl_json_response_builder!(EnglishWordInfoResponse);


#[utoipa::path(
    get,
    path = "/dictionary/english/{word_uuid}",
    tag = "dictionary:english",
    params(
        (
            "word_uuid" = String,
            Path,
            description = "UUID of the english word."
        )
    ),
    responses(
        (
            status = 200,
            description = "Information about the requested english word.",
            body = EnglishWordInfoResponse,
        ),
        (
            status = 400,
            description = "Invalid word UUID provided.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "Client error: invalid UUID." })
        ),
        (
            status = 404,
            description = "The requested english word does not exist."
        ),
        openapi::FailedAuthenticationResponses<openapi::RequiresWordRead>,
        openapi::InternalServerErrorResponse,
    )
)]
#[get("/{word_uuid}")]
pub async fn get_specific_english_word(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(String,)>,
) -> EndpointResult {
    require_permission_with_optional_authentication!(state, authentication, Permission::WordRead);


    let target_word_uuid = parse_string_into_uuid(&parameters.into_inner().0)?;


    let target_word = EnglishWordQuery::word_by_uuid(&state.database, target_word_uuid)
        .await
        .map_err(APIError::InternalError)?;

    let Some(target_word) = target_word else {
        return Err(APIError::not_found());
    };


    let translation_suggestions =
        TranslationSuggestionQuery::suggestions_for_english_word(&state.database, target_word_uuid)
            .await
            .map_err(APIError::InternalError)?;

    let translations =
        TranslationQuery::translations_for_english_word(&state.database, target_word_uuid)
            .await
            .map_err(APIError::InternalError)?;


    Ok(EnglishWordInfoResponse {
        word: EnglishWord::from_models(target_word, translation_suggestions, translations),
    }
    .into_response())
}


#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug, ToSchema)]
pub struct EnglishWordUpdateRequest {
    pub lemma: Option<String>,
    pub disambiguation: Option<String>,
    pub description: Option<String>,
}

impl_json_response_builder!(EnglishWordUpdateRequest);



#[utoipa::path(
    patch,
    path = "/dictionary/english/{word_uuid}",
    tag = "dictionary:english",
    params(
        (
            "word_uuid" = String,
            Path,
            description = "UUID of the english word."
        )
    ),
    request_body(
        content = EnglishWordUpdateRequest,
    ),
    responses(
        (
            status = 200,
            description = "Updated english word.",
            body = EnglishWordInfoResponse,
        ),
        (
            status = 400,
            description = "Invalid word UUID provided.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "Client error: invalid UUID." })
        ),
        (
            status = 404,
            description = "The requested english word does not exist."
        ),
        openapi::FailedAuthenticationResponses<openapi::RequiresWordUpdate>,
        openapi::InternalServerErrorResponse,
    )
)]
#[patch("/{word_uuid}")]
pub async fn update_specific_english_word(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(String,)>,
    request_data: web::Json<EnglishWordUpdateRequest>,
) -> EndpointResult {
    let authenticated_user = require_authentication!(authentication);
    require_permission!(state, authenticated_user, Permission::WordUpdate);


    let target_word_uuid = parse_string_into_uuid(&parameters.into_inner().0)?;

    let request_data = request_data.into_inner();


    let target_word_exists =
        EnglishWordQuery::word_exists_by_uuid(&state.database, target_word_uuid)
            .await
            .map_err(APIError::InternalError)?;

    if !target_word_exists {
        return Err(APIError::not_found());
    }


    let updated_model = EnglishWordMutation::update(
        &state.database,
        target_word_uuid,
        UpdatedEnglishWord {
            lemma: request_data.lemma,
            disambiguation: request_data.disambiguation,
            description: request_data.description,
        },
    )
    .await
    .map_err(APIError::InternalError)?;


    let translation_suggestions =
        TranslationSuggestionQuery::suggestions_for_english_word(&state.database, target_word_uuid)
            .await
            .map_err(APIError::InternalError)?;

    let translations =
        TranslationQuery::translations_for_english_word(&state.database, target_word_uuid)
            .await
            .map_err(APIError::InternalError)?;


    Ok(EnglishWordInfoResponse {
        word: EnglishWord::from_models(
            updated_model,
            translation_suggestions,
            translations,
        ),
    }
    .into_response())
}



#[utoipa::path(
    delete,
    path = "/dictionary/english/{word_uuid}",
    tag = "dictionary:english",
    params(
        (
            "word_uuid" = String,
            Path,
            description = "UUID of the english word to delete."
        )
    ),
    responses(
        (
            status = 200,
            description = "English word deleted.",
        ),
        (
            status = 400,
            description = "Invalid word UUID provided.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "Client error: invalid UUID." })
        ),
        (
            status = 404,
            description = "The given english word does not exist."
        ),
        openapi::FailedAuthenticationResponses<openapi::RequiresWordDelete>,
        openapi::InternalServerErrorResponse,
    )
)]
#[delete("/{word_uuid}")]
pub async fn delete_specific_english_word(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(String,)>,
) -> EndpointResult {
    let authenticated_user = require_authentication!(authentication);
    require_permission!(state, authenticated_user, Permission::WordDelete);


    let target_word_uuid = parse_string_into_uuid(&parameters.into_inner().0)?;

    let target_word_exists =
        EnglishWordQuery::word_exists_by_uuid(&state.database, target_word_uuid)
            .await
            .map_err(APIError::InternalError)?;

    if !target_word_exists {
        return Err(APIError::not_found());
    }


    EnglishWordMutation::delete(&state.database, target_word_uuid)
        .await
        .map_err(APIError::InternalError)?;


    Ok(HttpResponse::Ok().finish())
}



// TODO Links, suggestions, translations.


#[rustfmt::skip]
pub fn english_dictionary_router() -> Scope {
    web::scope("/english")
        .service(get_all_english_words)
        .service(create_english_word)
        .service(get_specific_english_word)
        .service(update_specific_english_word)
        .service(delete_specific_english_word)
}
