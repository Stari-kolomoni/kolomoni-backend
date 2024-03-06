use actix_http::StatusCode;
use actix_web::{delete, post, web, HttpResponse, Scope};
use kolomoni_auth::Permission;
use kolomoni_database::{
    mutation::{NewTranslation, TranslationMutation, TranslationToDelete},
    query::{EnglishWordQuery, SloveneWordQuery, TranslationQuery},
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
pub struct TranslationRequest {
    pub english_word_id: String,
    pub slovene_word_id: String,
}


/// Create a new translation
///
/// This endpoint will create a new translation relationship
/// between an english and a slovene word. Note that this is different than
/// a *translation suggestion*.
///
/// # Authentication
/// This endpoint requires authentication and the `word.translation:create` permission.
#[utoipa::path(
    post,
    path = "/dictionary/translation",
    tag = "dictionary:translation",
    request_body(
        content = TranslationRequest
    ),
    responses(
        (
            status = 200,
            description = "The translation has been created."
        ),
        (
            status = 400,
            description = "The provided slovene or english word does not exist.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "The provided english word does not exist." })
        ),
        (
            status = 409,
            description = "The translation already exists.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "The translation already exists." })
        ),
        openapi::MissingOrInvalidJsonRequestBodyResponse,
        openapi::FailedAuthenticationResponses<openapi::RequiresTranslationCreate>,
        openapi::InternalServerErrorResponse,
    ),
    security(
        ("access_token" = [])
    )
)]
#[post("")]
pub async fn create_translation(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    request_body: web::Json<TranslationRequest>,
) -> EndpointResult {
    let authenticated_user = require_authentication!(authentication);
    require_permission!(
        state,
        authenticated_user,
        Permission::TranslationCreate
    );


    let request_body = request_body.into_inner();

    let english_word_uuid = parse_string_into_uuid(&request_body.english_word_id)?;
    let slovene_word_uuid = parse_string_into_uuid(&request_body.slovene_word_id)?;


    let english_word_exists =
        EnglishWordQuery::word_exists_by_uuid(&state.database, english_word_uuid)
            .await
            .map_err(APIError::InternalError)?;
    if !english_word_exists {
        return Err(APIError::client_error(
            "The provided english word does not exist.",
        ));
    }

    let slovene_word_exists =
        SloveneWordQuery::word_exists_by_uuid(&state.database, slovene_word_uuid)
            .await
            .map_err(APIError::InternalError)?;
    if !slovene_word_exists {
        return Err(APIError::client_error(
            "The provided slovene word does not exist.",
        ));
    }

    let translation_already_exists = TranslationQuery::exists(
        &state.database,
        english_word_uuid,
        slovene_word_uuid,
    )
    .await
    .map_err(APIError::InternalError)?;

    if translation_already_exists {
        return Ok(error_response_with_reason!(
            StatusCode::CONFLICT,
            "The translation already exists."
        ));
    }

    TranslationMutation::create(
        &state.database,
        NewTranslation {
            english_word_id: english_word_uuid,
            slovene_word_id: slovene_word_uuid,
        },
    )
    .await
    .map_err(APIError::InternalError)?;



    // Signals to the search engine that both words have been updated.
    state
        .search
        .signal_english_word_created_or_updated(english_word_uuid)
        .await
        .map_err(APIError::InternalError)?;
    state
        .search
        .signal_slovene_word_created_or_updated(slovene_word_uuid)
        .await
        .map_err(APIError::InternalError)?;


    Ok(HttpResponse::Ok().finish())
}




#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct TranslationDeletionRequest {
    pub english_word_id: String,
    pub slovene_word_id: String,
}



/// Delete a translation
///
/// This endpoint will remove a translation relationship
/// between an english and a slovene word. Note that this is different than
/// a *translation suggestion*.
///
/// # Authentication
/// This endpoint requires authentication and the `word.translation:delete` permission.
#[utoipa::path(
    delete,
    path = "/dictionary/translation",
    tag = "dictionary:translation",
    request_body(
        content = TranslationDeletionRequest
    ),
    responses(
        (
            status = 200,
            description = "The translation relationship has been deleted."
        ),
        (
            status = 400,
            description = "The provided slovene or english word does not exist.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "The provided english word does not exist." })
        ),
        (
            status = 404,
            description = "The translation relationship does not exist.",
        ),
        openapi::MissingOrInvalidJsonRequestBodyResponse,
        openapi::FailedAuthenticationResponses<openapi::RequiresTranslationDelete>,
        openapi::InternalServerErrorResponse,
    ),
    security(
        ("access_token" = [])
    )
)]
#[delete("")]
pub async fn delete_translation(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    request_body: web::Json<TranslationDeletionRequest>,
) -> EndpointResult {
    let authenticated_user = require_authentication!(authentication);
    require_permission!(
        state,
        authenticated_user,
        Permission::TranslationDelete
    );


    let request_body = request_body.into_inner();

    let english_word_uuid = parse_string_into_uuid(&request_body.english_word_id)?;
    let slovene_word_uuid = parse_string_into_uuid(&request_body.slovene_word_id)?;


    let english_word_exists =
        EnglishWordQuery::word_exists_by_uuid(&state.database, english_word_uuid)
            .await
            .map_err(APIError::InternalError)?;
    if !english_word_exists {
        return Err(APIError::client_error(
            "The provided english word does not exist.",
        ));
    }

    let slovene_word_exists =
        SloveneWordQuery::word_exists_by_uuid(&state.database, slovene_word_uuid)
            .await
            .map_err(APIError::InternalError)?;
    if !slovene_word_exists {
        return Err(APIError::client_error(
            "The provided slovene word does not exist.",
        ));
    }

    let suggestion_exists = TranslationQuery::exists(
        &state.database,
        english_word_uuid,
        slovene_word_uuid,
    )
    .await
    .map_err(APIError::InternalError)?;

    if !suggestion_exists {
        return Err(APIError::not_found());
    }


    TranslationMutation::delete(
        &state.database,
        TranslationToDelete {
            english_word_id: english_word_uuid,
            slovene_word_id: slovene_word_uuid,
        },
    )
    .await
    .map_err(APIError::InternalError)?;



    // Signals to the search engine that both words have been updated.
    state
        .search
        .signal_english_word_created_or_updated(english_word_uuid)
        .await
        .map_err(APIError::InternalError)?;
    state
        .search
        .signal_slovene_word_created_or_updated(slovene_word_uuid)
        .await
        .map_err(APIError::InternalError)?;


    Ok(HttpResponse::Ok().finish())
}


#[rustfmt::skip]
pub fn translations_router() -> Scope {
    web::scope("/translation")
        .service(create_translation)
        .service(delete_translation)
}
