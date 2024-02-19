use std::str::FromStr;

use actix_http::StatusCode;
use actix_web::{delete, get, patch, post, web, HttpResponse, Scope};
use chrono::{DateTime, Utc};
use kolomoni_auth::Permission;
use kolomoni_database::{
    entities,
    mutation::{EnglishWordMutation, NewEnglishWord, UpdatedEnglishWord},
    query::{self, EnglishWordQuery},
};
use miette::IntoDiagnostic;
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    api::{
        errors::{APIError, EndpointResult},
        macros::ContextlessResponder,
        openapi,
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
    pub word_id: String,
    pub lemma: String,
    pub disambiguation: Option<String>,
    pub description: Option<String>,
    pub added_at: DateTime<Utc>,
    pub last_edited_at: DateTime<Utc>,
}

impl EnglishWord {
    pub fn from_database_model(model: entities::word_english::Model) -> Self {
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

    let words_as_api_structures = words
        .into_iter()
        .map(EnglishWord::from_database_model)
        .collect();


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
        word: EnglishWord::from_database_model(newly_created_word),
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


    let target_word_uuid_string = parameters.into_inner().0;
    let target_word_uuid = Uuid::from_str(&target_word_uuid_string)
        .into_diagnostic()
        .map_err(|_| APIError::client_error("invalid UUID"))?;


    let target_word = EnglishWordQuery::word_by_uuid(&state.database, target_word_uuid)
        .await
        .map_err(APIError::InternalError)?;

    let Some(target_word) = target_word else {
        return Err(APIError::not_found());
    };


    Ok(EnglishWordInfoResponse {
        word: EnglishWord::from_database_model(target_word),
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


    let target_word_uuid_string = parameters.into_inner().0;
    let target_word_uuid = Uuid::from_str(&target_word_uuid_string)
        .into_diagnostic()
        .map_err(|_| APIError::client_error("invalid UUID"))?;

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


    Ok(EnglishWordInfoResponse {
        word: EnglishWord::from_database_model(updated_model),
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


    let target_word_uuid_string = parameters.into_inner().0;
    let target_word_uuid = Uuid::from_str(&target_word_uuid_string)
        .into_diagnostic()
        .map_err(|_| APIError::client_error("invalid UUID"))?;


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
