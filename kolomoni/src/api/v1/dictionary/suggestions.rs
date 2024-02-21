use actix_http::StatusCode;
use actix_web::{delete, post, web, HttpResponse, Scope};
use kolomoni_auth::Permission;
use kolomoni_database::{
    mutation::{
        NewTranslationSuggestion,
        TranslationSuggestionMutation,
        TranslationSuggestionToDelete,
    },
    query::TranslationSuggestionQuery,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    api::{
        errors::{APIError, EndpointResult},
        openapi,
        v1::dictionary::parse_string_into_uuid,
    },
    authentication::UserAuthenticationExtractor,
    error_response_with_reason,
    require_authentication,
    require_permission,
    state::ApplicationState,
};


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct TranslationSuggestionRequest {
    english_word_id: String,
    slovene_word_id: String,
}


#[utoipa::path(
    post,
    path = "/suggestion",
    tag = "dictionary:suggestion",
    request_body(
        content = TranslationSuggestionRequest
    ),
    responses(
        (
            status = 200,
            description = "The translation suggestion relationship has been created."
        ),
        (
            status = 409,
            description = "The translation suggestion relationship already exists.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "The translation suggestion already exists." })
        ),
        openapi::FailedAuthenticationResponses<openapi::RequiresSuggestionCreate>,
        openapi::InternalServerErrorResponse,
    )
)]
#[post("")]
pub async fn suggest_translation(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    request_body: web::Json<TranslationSuggestionRequest>,
) -> EndpointResult {
    let authenticated_user = require_authentication!(authentication);
    require_permission!(
        state,
        authenticated_user,
        Permission::SuggestionCreate
    );


    let request_body = request_body.into_inner();

    let english_word_uuid = parse_string_into_uuid(&request_body.english_word_id)?;
    let slovene_word_uuid = parse_string_into_uuid(&request_body.slovene_word_id)?;


    let suggestion_already_exists = TranslationSuggestionQuery::exists(
        &state.database,
        english_word_uuid,
        slovene_word_uuid,
    )
    .await
    .map_err(APIError::InternalError)?;

    if suggestion_already_exists {
        return Ok(error_response_with_reason!(
            StatusCode::CONFLICT,
            "The translation suggestion already exists."
        ));
    }


    TranslationSuggestionMutation::create(
        &state.database,
        NewTranslationSuggestion {
            english_word_id: english_word_uuid,
            slovene_word_id: slovene_word_uuid,
        },
    )
    .await
    .map_err(APIError::InternalError)?;


    Ok(HttpResponse::Ok().finish())
}




#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct TranslationSuggestionDeletionRequest {
    english_word_id: String,
    slovene_word_id: String,
}

#[utoipa::path(
    delete,
    path = "/suggestion",
    tag = "dictionary:suggestion",
    request_body(
        content = TranslationSuggestionDeletionRequest
    ),
    responses(
        (
            status = 200,
            description = "The translation suggestion relationship has been deleted."
        ),
        (
            status = 404,
            description = "The translation suggestion relationship does not exist.",
        ),
        openapi::FailedAuthenticationResponses<openapi::RequiresSuggestionDelete>,
        openapi::InternalServerErrorResponse,
    )
)]
#[delete("")]
pub async fn delete_suggestion(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    request_body: web::Json<TranslationSuggestionDeletionRequest>,
) -> EndpointResult {
    let authenticated_user = require_authentication!(authentication);
    require_permission!(
        state,
        authenticated_user,
        Permission::SuggestionDelete
    );


    let request_body = request_body.into_inner();

    let english_word_uuid = parse_string_into_uuid(&request_body.english_word_id)?;
    let slovene_word_uuid = parse_string_into_uuid(&request_body.slovene_word_id)?;


    let suggestion_exists = TranslationSuggestionQuery::exists(
        &state.database,
        english_word_uuid,
        slovene_word_uuid,
    )
    .await
    .map_err(APIError::InternalError)?;

    if !suggestion_exists {
        return Err(APIError::not_found());
    }


    TranslationSuggestionMutation::delete(
        &state.database,
        TranslationSuggestionToDelete {
            english_word_id: english_word_uuid,
            slovene_word_id: slovene_word_uuid,
        },
    )
    .await
    .map_err(APIError::InternalError)?;


    Ok(HttpResponse::Ok().finish())
}


#[rustfmt::skip]
pub fn suggested_translations_router() -> Scope {
    web::scope("/suggestion")
        .service(suggest_translation)
        .service(delete_suggestion)
}
