use actix_http::StatusCode;
use actix_web::{delete, post, web, HttpResponse, Scope};
use kolomoni_auth::Permission;
use kolomoni_database::{
    mutation::{
        NewTranslationSuggestion,
        TranslationSuggestionMutation,
        TranslationSuggestionToDelete,
    },
    query::{EnglishWordQuery, SloveneWordQuery, TranslationSuggestionQuery},
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
    json_error_response_with_reason,
    require_authentication,
    require_permission,
    state::ApplicationState,
};

// TODO this module needs to be removed


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct TranslationSuggestionRequest {
    pub english_word_id: String,
    pub slovene_word_id: String,
}



/// Create a new translation suggestion
///
/// This endpoint will create a new translation suggestion relationship
/// between an english and a slovene word.
///
/// # Authentication
/// This endpoint requires authentication and the `word.suggestion:create` permission.
#[utoipa::path(
    post,
    path = "/dictionary/suggestion",
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
            status = 400,
            description = "The provided slovene or english word does not exist.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "The provided english word does not exist." })
        ),
        (
            status = 409,
            description = "The translation suggestion relationship already exists.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "The translation suggestion already exists." })
        ),
        openapi::MissingOrInvalidJsonRequestBodyResponse,
        openapi::FailedAuthenticationResponses<openapi::RequiresSuggestionCreate>,
        openapi::InternalServerErrorResponse,
    ),
    security(
        ("access_token" = [])
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


    let english_word_exists =
        EnglishWordQuery::word_exists_by_uuid(&state.database, english_word_uuid)
            .await
            .map_err(APIError::InternalGenericError)?;
    if !english_word_exists {
        return Err(APIError::client_error(
            "The provided english word does not exist.",
        ));
    }

    let slovene_word_exists =
        SloveneWordQuery::word_exists_by_uuid(&state.database, slovene_word_uuid)
            .await
            .map_err(APIError::InternalGenericError)?;
    if !slovene_word_exists {
        return Err(APIError::client_error(
            "The provided slovene word does not exist.",
        ));
    }


    let suggestion_already_exists = TranslationSuggestionQuery::exists(
        &state.database,
        english_word_uuid,
        slovene_word_uuid,
    )
    .await
    .map_err(APIError::InternalGenericError)?;

    if suggestion_already_exists {
        return Ok(json_error_response_with_reason!(
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
    .map_err(APIError::InternalGenericError)?;



    // Signals to the search engine that both words have been updated.
    state
        .search
        .signal_english_word_created_or_updated(english_word_uuid)
        .await
        .map_err(APIError::InternalGenericError)?;
    state
        .search
        .signal_slovene_word_created_or_updated(slovene_word_uuid)
        .await
        .map_err(APIError::InternalGenericError)?;


    Ok(HttpResponse::Ok().finish())
}




#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct TranslationSuggestionDeletionRequest {
    pub english_word_id: String,
    pub slovene_word_id: String,
}



/// Delete a translation suggestion
///
/// This endpoint will remove a translation suggestion relationship
/// between an english and a slovene word.
///
/// # Authentication
/// This endpoint requires authentication and the `word.suggestion:delete` permission.
#[utoipa::path(
    delete,
    path = "/dictionary/suggestion",
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
            status = 400,
            description = "The provided slovene or english word does not exist.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "The provided english word does not exist." })
        ),
        (
            status = 404,
            description = "The translation suggestion relationship does not exist.",
        ),
        openapi::MissingOrInvalidJsonRequestBodyResponse,
        openapi::FailedAuthenticationResponses<openapi::RequiresSuggestionDelete>,
        openapi::InternalServerErrorResponse,
    ),
    security(
        ("access_token" = [])
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



    let english_word_exists =
        EnglishWordQuery::word_exists_by_uuid(&state.database, english_word_uuid)
            .await
            .map_err(APIError::InternalGenericError)?;
    if !english_word_exists {
        return Err(APIError::client_error(
            "The provided english word does not exist.",
        ));
    }

    let slovene_word_exists =
        SloveneWordQuery::word_exists_by_uuid(&state.database, slovene_word_uuid)
            .await
            .map_err(APIError::InternalGenericError)?;
    if !slovene_word_exists {
        return Err(APIError::client_error(
            "The provided slovene word does not exist.",
        ));
    }



    let suggestion_exists = TranslationSuggestionQuery::exists(
        &state.database,
        english_word_uuid,
        slovene_word_uuid,
    )
    .await
    .map_err(APIError::InternalGenericError)?;

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
    .map_err(APIError::InternalGenericError)?;



    // Signals to the search engine that both words have been updated.
    state
        .search
        .signal_english_word_created_or_updated(english_word_uuid)
        .await
        .map_err(APIError::InternalGenericError)?;
    state
        .search
        .signal_slovene_word_created_or_updated(slovene_word_uuid)
        .await
        .map_err(APIError::InternalGenericError)?;


    Ok(HttpResponse::Ok().finish())
}


#[rustfmt::skip]
pub fn suggested_translations_router() -> Scope {
    web::scope("/suggestion")
        .service(suggest_translation)
        .service(delete_suggestion)
}
