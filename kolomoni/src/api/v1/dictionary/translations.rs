use actix_web::{delete, post, web, Scope};
use kolomoni_auth::Permission;
use kolomoni_core::{
    api_models::{TranslationCreationRequest, TranslationDeletionRequest},
    id::{EnglishWordMeaningId, SloveneWordMeaningId},
};
use kolomoni_database::entities;
use sqlx::Acquire;
use tracing::info;

use crate::{
    api::{
        errors::{EndpointError, EndpointResponseBuilder, EndpointResult, TranslationsErrorReason},
        openapi,
    },
    authentication::UserAuthenticationExtractor,
    require_user_authentication_and_permission,
    state::ApplicationState,
};




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
        openapi::response::MissingOrInvalidJsonRequestBody,
        openapi::response::FailedAuthentication<openapi::response::requires::TranslationCreate>,
        openapi::response::InternalServerError,
    ),
    security(
        ("access_token" = [])
    )
)]
#[post("")]
pub async fn create_translation(
    state: ApplicationState,
    authentication_extractor: UserAuthenticationExtractor,
    request_body: web::Json<TranslationCreationRequest>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;
    let mut transaction = database_connection.begin().await?;

    let authenticated_user = require_user_authentication_and_permission!(
        &mut transaction,
        authentication_extractor,
        Permission::TranslationCreate
    );


    let request_body = request_body.into_inner();

    let english_word_meaning_id = EnglishWordMeaningId::new(request_body.english_word_meaning_id);
    let slovene_word_meaning_id = SloveneWordMeaningId::new(request_body.slovene_word_meaning_id);



    let english_word_exists =
        entities::EnglishWordMeaningQuery::exists_by_id(&mut transaction, english_word_meaning_id)
            .await?;

    if !english_word_exists {
        return EndpointResponseBuilder::not_found()
            .with_error_reason(TranslationsErrorReason::english_word_meaning_not_found())
            .build();
    }


    let slovene_word_exists =
        entities::SloveneWordMeaningQuery::exists_by_id(&mut transaction, slovene_word_meaning_id)
            .await?;

    if !slovene_word_exists {
        return EndpointResponseBuilder::not_found()
            .with_error_reason(TranslationsErrorReason::slovene_word_meaning_not_found())
            .build();
    }



    let translation_already_exists = entities::WordMeaningTranslationQuery::exists(
        &mut transaction,
        english_word_meaning_id,
        slovene_word_meaning_id,
    )
    .await?;

    if translation_already_exists {
        return EndpointResponseBuilder::conflict()
            .with_error_reason(TranslationsErrorReason::translation_relationship_already_exists())
            .build();
    }


    let _ = entities::WordMeaningTranslationMutation::create(
        &mut transaction,
        english_word_meaning_id,
        slovene_word_meaning_id,
        Some(authenticated_user.user_id()),
    )
    .await?;



    /* TODO pending cache layer rewrite
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
        .map_err(APIError::InternalGenericError)?; */


    EndpointResponseBuilder::ok().build()
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
        openapi::response::MissingOrInvalidJsonRequestBody,
        openapi::response::FailedAuthentication<openapi::response::requires::TranslationDelete>,
        openapi::response::InternalServerError,
    ),
    security(
        ("access_token" = [])
    )
)]
#[delete("")]
pub async fn delete_translation(
    state: ApplicationState,
    authentication_extractor: UserAuthenticationExtractor,
    request_body: web::Json<TranslationDeletionRequest>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;
    let mut transaction = database_connection.begin().await?;

    let authenticated_user = require_user_authentication_and_permission!(
        &mut transaction,
        authentication_extractor,
        Permission::TranslationDelete
    );


    let request_body = request_body.into_inner();

    let english_word_meaning_id = EnglishWordMeaningId::new(request_body.english_word_meaning_id);
    let slovene_word_meaning_id = SloveneWordMeaningId::new(request_body.slovene_word_meaning_id);


    let english_word_meaning_exists =
        entities::EnglishWordMeaningQuery::exists_by_id(&mut transaction, english_word_meaning_id)
            .await?;

    if !english_word_meaning_exists {
        // FIXME fix docs, status code changed here
        return EndpointResponseBuilder::not_found()
            .with_error_reason(TranslationsErrorReason::english_word_meaning_not_found())
            .build();
    }


    let slovene_word_meaning_exists =
        entities::SloveneWordMeaningQuery::exists_by_id(&mut transaction, slovene_word_meaning_id)
            .await?;

    if !slovene_word_meaning_exists {
        // FIXME fix docs, status code changed here
        return EndpointResponseBuilder::not_found()
            .with_error_reason(TranslationsErrorReason::slovene_word_meaning_not_found())
            .build();
    }


    let translation_relationship_exists = entities::WordMeaningTranslationQuery::exists(
        &mut transaction,
        english_word_meaning_id,
        slovene_word_meaning_id,
    )
    .await?;

    if !translation_relationship_exists {
        return EndpointResponseBuilder::not_found()
            .with_error_reason(TranslationsErrorReason::translation_relationship_not_found())
            .build();
    }


    let deleted_translation_relationship_successfully =
        entities::WordMeaningTranslationMutation::delete(
            &mut transaction,
            english_word_meaning_id,
            slovene_word_meaning_id,
        )
        .await?;


    if !deleted_translation_relationship_successfully {
        return Err(EndpointError::internal_error_with_reason(
            "database inconsistency: failed to delete a translation relationship \
            even though it previously existed inside the same transaction",
        ));
    }


    info!(
        operator = %authenticated_user.user_id(),
        "Deleted translation relationship: {} <-> {}",
        english_word_meaning_id, slovene_word_meaning_id
    );



    /* TODO pending cache layer rewrite
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
        .map_err(APIError::InternalGenericError)?; */


    EndpointResponseBuilder::ok().build()
}




#[rustfmt::skip]
pub fn translations_router() -> Scope {
    web::scope("/translation")
        .service(create_translation)
        .service(delete_translation)
}
