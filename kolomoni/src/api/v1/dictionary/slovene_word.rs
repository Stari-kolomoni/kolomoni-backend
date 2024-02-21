use actix_http::StatusCode;
use actix_web::{delete, get, patch, post, web, HttpResponse, Scope};
use chrono::{DateTime, Utc};
use kolomoni_auth::Permission;
use kolomoni_database::{
    entities,
    mutation::{NewSloveneWord, SloveneWordMutation, UpdatedSloveneWord},
    query::{self, SloveneWordQuery},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
        "lemma": "pustolovec",
        "disambiguation": "lik",
        "description": "Igrani ali neigrani liki, ki se odpravijo na pustolovščino.",
        "added_at": "2023-06-27T20:34:27.217273Z",
        "last_edited_at": "2023-06-27T20:34:27.217273Z"
    })
)]
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
pub struct SloveneWordsResponse {
    slovene_words: Vec<SloveneWord>,
}

impl_json_response_builder!(SloveneWordsResponse);


#[utoipa::path(
    get,
    path = "/dictionary/slovene",
    tag = "dictionary:slovene",
    responses(
        (
            status = 200,
            description = "A list of all slovene words.",
            body = SloveneWordsResponse,
        ),
        openapi::FailedAuthenticationResponses<openapi::RequiresWordRead>,
        openapi::InternalServerErrorResponse,
    )
)]
#[get("")]
pub async fn get_all_slovene_words(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
) -> EndpointResult {
    require_permission_with_optional_authentication!(state, authentication, Permission::WordRead);

    // Load words from the database.
    let words = query::SloveneWordQuery::all_words(&state.database)
        .await
        .map_err(APIError::InternalError)?;

    let words_as_api_structures = words
        .into_iter()
        .map(SloveneWord::from_database_model)
        .collect();

    Ok(SloveneWordsResponse {
        slovene_words: words_as_api_structures,
    }
    .into_response())
}


#[derive(Deserialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Serialize))]
#[schema(
    example = json!({
        "lemma": "pustolovec",
        "disambiguation": "lik",
        "description": "Igrani ali neigrani liki, ki se odpravijo na pustolovščino."
    })
)]
pub struct SloveneWordCreationRequest {
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
            "lemma": "pustolovec",
            "disambiguation": "lik",
            "description": "Igrani ali neigrani liki, ki se odpravijo na pustolovščino.",
            "added_at": "2023-06-27T20:34:27.217273Z",
            "last_edited_at": "2023-06-27T20:34:27.217273Z"
        }
    })
)]
pub struct SloveneWordCreationResponse {
    pub word: SloveneWord,
}

impl_json_response_builder!(SloveneWordCreationResponse);


#[utoipa::path(
    post,
    path = "/dictionary/slovene",
    tag = "dictionary:slovene",
    request_body(
        content = SloveneWordCreationRequest
    ),
    responses(
        (
            status = 200,
            description = "The newly-created slovene word.",
            body = SloveneWordCreationResponse,
        ),
        (
            status = 409,
            description = "Slovene word with the given lemma already exists.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "A slovene word with the given lemma already exists." })
        ),
        openapi::FailedAuthenticationResponses<openapi::RequiresWordCreate>,
        openapi::InternalServerErrorResponse,
    )
)]
#[post("")]
pub async fn create_slovene_word(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    creation_request: web::Json<SloveneWordCreationRequest>,
) -> EndpointResult {
    let authenticated_user = require_authentication!(authentication);
    require_permission!(state, authenticated_user, Permission::WordCreate);


    let creation_request = creation_request.into_inner();

    let lemma_already_exists =
        SloveneWordQuery::word_exists_by_lemma(&state.database, creation_request.lemma.clone())
            .await
            .map_err(APIError::InternalError)?;

    if lemma_already_exists {
        return Ok(error_response_with_reason!(
            StatusCode::CONFLICT,
            "A slovene word with the given lemma already exists."
        ));
    }


    let newly_created_word = SloveneWordMutation::create(
        &state.database,
        NewSloveneWord {
            lemma: creation_request.lemma,
            disambiguation: creation_request.disambiguation,
            description: creation_request.description,
        },
    )
    .await
    .map_err(APIError::InternalError)?;


    Ok(SloveneWordCreationResponse {
        word: SloveneWord::from_database_model(newly_created_word),
    }
    .into_response())
}




#[derive(Serialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Deserialize))]
pub struct SloveneWordInfoResponse {
    pub word: SloveneWord,
}

impl_json_response_builder!(SloveneWordInfoResponse);


#[utoipa::path(
    get,
    path = "/dictionary/slovene/{word_uuid}",
    tag = "dictionary:slovene",
    params(
        (
            "word_uuid" = String,
            Path,
            description = "UUID of the slovene word."
        )
    ),
    responses(
        (
            status = 200,
            description = "Information about the requested slovene word.",
            body = SloveneWordInfoResponse,
        ),
        (
            status = 400,
            description = "Invalid word UUID provided.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "Client error: invalid UUID." })
        ),
        (
            status = 404,
            description = "The requested slovene word does not exist."
        ),
        openapi::FailedAuthenticationResponses<openapi::RequiresWordRead>,
        openapi::InternalServerErrorResponse,
    )
)]
#[get("/{word_uuid}")]
pub async fn get_specific_slovene_word(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(String,)>,
) -> EndpointResult {
    require_permission_with_optional_authentication!(state, authentication, Permission::WordRead);


    let target_word_uuid = parse_string_into_uuid(&parameters.into_inner().0)?;


    let target_word = SloveneWordQuery::word_by_uuid(&state.database, target_word_uuid)
        .await
        .map_err(APIError::InternalError)?;

    let Some(target_word) = target_word else {
        return Err(APIError::not_found());
    };


    Ok(SloveneWordInfoResponse {
        word: SloveneWord::from_database_model(target_word),
    }
    .into_response())
}



#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug, ToSchema)]
pub struct SloveneWordUpdateRequest {
    pub lemma: Option<String>,
    pub disambiguation: Option<String>,
    pub description: Option<String>,
}

impl_json_response_builder!(SloveneWordUpdateRequest);


#[utoipa::path(
    patch,
    path = "/dictionary/slovene/{word_uuid}",
    tag = "dictionary:slovene",
    params(
        (
            "word_uuid" = String,
            Path,
            description = "UUID of the slovene word."
        )
    ),
    request_body(
        content = SloveneWordUpdateRequest,
    ),
    responses(
        (
            status = 200,
            description = "Updated slovene word.",
            body = SloveneWordInfoResponse,
        ),
        (
            status = 400,
            description = "Invalid word UUID provided.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "Client error: invalid UUID." })
        ),
        (
            status = 404,
            description = "The requested slovene word does not exist."
        ),
        openapi::FailedAuthenticationResponses<openapi::RequiresWordUpdate>,
        openapi::InternalServerErrorResponse,
    )
)]
#[patch("/{word_uuid}")]
pub async fn update_specific_slovene_word(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(String,)>,
    request_data: web::Json<SloveneWordUpdateRequest>,
) -> EndpointResult {
    let authenticated_user = require_authentication!(authentication);
    require_permission!(state, authenticated_user, Permission::WordUpdate);


    let target_word_uuid = parse_string_into_uuid(&parameters.into_inner().0)?;

    let request_data = request_data.into_inner();


    let target_word_exists =
        SloveneWordQuery::word_exists_by_uuid(&state.database, target_word_uuid)
            .await
            .map_err(APIError::InternalError)?;

    if !target_word_exists {
        return Err(APIError::not_found());
    }


    let updated_word = SloveneWordMutation::update(
        &state.database,
        target_word_uuid,
        UpdatedSloveneWord {
            lemma: request_data.lemma,
            disambiguation: request_data.disambiguation,
            description: request_data.description,
        },
    )
    .await
    .map_err(APIError::InternalError)?;


    Ok(SloveneWordInfoResponse {
        word: SloveneWord::from_database_model(updated_word),
    }
    .into_response())
}



#[utoipa::path(
    delete,
    path = "/dictionary/slovene/{word_uuid}",
    tag = "dictionary:slovene",
    params(
        (
            "word_uuid" = String,
            Path,
            description = "UUID of the slovene word to delete."
        )
    ),
    responses(
        (
            status = 200,
            description = "Slovene word deleted.",
        ),
        (
            status = 400,
            description = "Invalid word UUID provided.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "Client error: invalid UUID." })
        ),
        (
            status = 404,
            description = "The given slovene word does not exist."
        ),
        openapi::FailedAuthenticationResponses<openapi::RequiresWordDelete>,
        openapi::InternalServerErrorResponse,
    )
)]
#[delete("/{word_uuid}")]
pub async fn delete_specific_slovene_word(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(String,)>,
) -> EndpointResult {
    let authenticated_user = require_authentication!(authentication);
    require_permission!(state, authenticated_user, Permission::WordDelete);


    let target_word_uuid = parse_string_into_uuid(&parameters.into_inner().0)?;


    let target_word_exists =
        SloveneWordQuery::word_exists_by_uuid(&state.database, target_word_uuid)
            .await
            .map_err(APIError::InternalError)?;

    if !target_word_exists {
        return Err(APIError::not_found());
    }


    SloveneWordMutation::delete(&state.database, target_word_uuid)
        .await
        .map_err(APIError::InternalError)?;


    Ok(HttpResponse::Ok().finish())
}


// TODO Links, suggestions, translations.


#[rustfmt::skip]
pub fn slovene_dictionary_router() -> Scope {
    web::scope("/slovene")
        .service(get_all_slovene_words)
        .service(create_slovene_word)
        .service(get_specific_slovene_word)
        .service(update_specific_slovene_word)
        .service(delete_specific_slovene_word)
}
