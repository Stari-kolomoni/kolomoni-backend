use actix_web::{delete, get, patch, post, web, HttpResponse, Scope};
use kolomoni_core::api_models::WordErrorReason;
use kolomoni_core::permissions::Permission;
use kolomoni_core::{
    api_models::{
        EnglishWordMeaningUpdateRequest,
        EnglishWordMeaningUpdatedResponse,
        EnglishWordMeaningsResponse,
        NewEnglishWordMeaningCreatedResponse,
        NewEnglishWordMeaningRequest,
    },
    ids::{EnglishWordId, EnglishWordMeaningId},
};
use kolomoni_database::entities::{
    self,
    EnglishWordMeaningLookup,
    EnglishWordMeaningUpdate,
    NewEnglishWordMeaning,
};

use crate::api::openapi;
use crate::api::openapi::response::{requires, AsErrorReason};
use crate::api::v1::dictionary::english::EnglishWordNotFound;
use crate::api::v1::dictionary::parse_uuid;
use crate::declare_openapi_error_reason_response;
use crate::{
    api::{
        errors::{EndpointError, EndpointResponseBuilder, EndpointResult},
        traits::IntoApiModel,
    },
    authentication::UserAuthenticationExtractor,
    require_permission_with_optional_authentication,
    require_user_authentication_and_permissions,
    state::ApplicationState,
};




/// Get all word meanings for a given english word
///
/// This endpoint returns a list of all word meanings
/// that the specified english word has.
///
///
/// # Authentication & Required permissions
/// - Authentication **is not** required.
/// - The caller must have the `word:read` permission, which is currently
///   blanket-granted to both unauthenticated and authenticated users.
#[utoipa::path(
    get,
    path = "/dictionary/english/{english_word_id}/meaning",
    tag = "dictionary:english:meaning",
    params(
        (
            "english_word_id" = String,
            Path,
            format = Uuid,
            description = "UUID of the english word to get meanings for."
        )
    ),
    responses(
        (
            status = 200,
            description = "Requested english word meanings.",
            body = EnglishWordMeaningsResponse
        ),
        (
            status = 404,
            response = inline(AsErrorReason<EnglishWordNotFound>)
        ),
        openapi::response::UuidUrlParameterError,
        openapi::response::MissingPermissions<requires::WordRead, 1>,
        openapi::response::InternalServerError,
    )
)]
#[get("")]
pub async fn get_all_english_word_meanings(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(String,)>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;

    require_permission_with_optional_authentication!(
        &mut database_connection,
        authentication,
        Permission::WordRead
    );


    let target_english_word_id = parse_uuid::<EnglishWordId>(parameters.into_inner().0)?;


    let english_word_exists =
        entities::EnglishWordQuery::exists_by_id(&mut database_connection, target_english_word_id)
            .await?;

    if !english_word_exists {
        return EndpointResponseBuilder::not_found()
            .with_error_reason(WordErrorReason::word_not_found())
            .build();
    }


    let english_word_meanings = entities::EnglishWordMeaningQuery::get_all_by_english_word_id(
        &mut database_connection,
        target_english_word_id,
    )
    .await?;


    EndpointResponseBuilder::ok()
        .with_json_body(EnglishWordMeaningsResponse {
            meanings: english_word_meanings
                .into_iter()
                .map(|meaning| meaning.into_api_model())
                .collect(),
        })
        .build()
}


declare_openapi_error_reason_response!(
    pub struct EnglishWordMeaningAlreadyExists {
        description => "An english word meaning with the given fields already exists.",
        reason => WordErrorReason::identical_word_meaning_already_exists()
    }
);



/// Create a new english word meaning
///
/// This endpoint creates a new english word meaning with
/// the given disambiguation, abbreviation, and description.
///
/// Just to clarify: word meanings are *always* linked
/// to specified words, and cannot exist by themselves.
///
///
/// # Authentication & Required permissions
/// - Authentication **is** required.
/// - The caller must have the `word:update` permission.
#[utoipa::path(
    post,
    path = "/dictionary/english/{english_word_id}/meaning",
    tag = "dictionary:english:meaning",
    params(
        (
            "english_word_id" = String,
            Path,
            format = Uuid,
            description = "UUID of the english word to associate the meaning with."
        )
    ),
    request_body(
        content = NewEnglishWordMeaningRequest
    ),
    responses(
        (
            status = 200,
            description = "Newly-created english word meaning.",
            body = NewEnglishWordMeaningCreatedResponse
        ),
        (
            status = 409,
            response = inline(AsErrorReason<EnglishWordMeaningAlreadyExists>)
        ),
        openapi::response::UuidUrlParameterError,
        openapi::response::MissingAuthentication,
        openapi::response::MissingPermissions<requires::WordUpdate, 1>,
        openapi::response::InternalServerError,
    )
)]
#[post("")]
pub async fn create_english_word_meaning(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(String,)>,
    request_data: web::Json<NewEnglishWordMeaningRequest>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;
    let mut transaction = database_connection.transaction().begin().await?;

    require_user_authentication_and_permissions!(
        &mut transaction,
        authentication,
        Permission::WordUpdate
    );


    let target_english_word_id = parse_uuid::<EnglishWordId>(parameters.into_inner().0)?;

    let new_word_meaning_data = request_data.into_inner();


    let identical_meaning_already_exists =
        entities::EnglishWordMeaningQuery::exists_by_distinguishing_fields(
            &mut transaction,
            EnglishWordMeaningLookup {
                abbreviation: new_word_meaning_data.abbreviation.clone(),
                disambiguation: new_word_meaning_data.disambiguation.clone(),
                description: new_word_meaning_data.description.clone(),
            },
        )
        .await?;

    if identical_meaning_already_exists {
        return EndpointResponseBuilder::conflict()
            .with_error_reason(WordErrorReason::identical_word_meaning_already_exists())
            .build();
    }


    let newly_created_meaning = entities::EnglishWordMeaningMutation::create(
        &mut transaction,
        target_english_word_id,
        NewEnglishWordMeaning {
            abbreviation: new_word_meaning_data.abbreviation,
            description: new_word_meaning_data.description,
            disambiguation: new_word_meaning_data.disambiguation,
        },
    )
    .await?;

    transaction.commit().await?;


    EndpointResponseBuilder::ok()
        .with_json_body(NewEnglishWordMeaningCreatedResponse {
            meaning: newly_created_meaning.into_api_model(),
        })
        .build()
}



declare_openapi_error_reason_response!(
    pub struct EnglishWordMeaningNotFound {
        description => "The english word exists, but its associated word meaning does not.",
        reason => WordErrorReason::word_meaning_not_found()
    }
);



/// Modifies an english word meaning
///
/// This endpoint modifies an english word meaning.
///
///
/// # Double option
/// Note the use of double options - leaving a field undefined
/// semantically means "leave it alone", while setting it to `null`
/// semantically means "clear the field".
///
///
/// # Authentication & Required permissions
/// - Authentication **is** required.
/// - The caller must have the `word:update` permission.
#[utoipa::path(
    patch,
    path = "/dictionary/english/{english_word_id}/meaning/{english_word_meaning_id}",
    tag = "dictionary:english:meaning",
    params(
        (
            "english_word_id" = String,
            Path,
            format = Uuid,
            description = "UUID of the english word related to the meaning."
        ),
        (
            "english_word_meaning_id" = String,
            Path,
            format = Uuid,
            description = "UUID of the english word meaning to modify."
        )
    ),
    request_body(
        content = EnglishWordMeaningUpdateRequest
    ),
    responses(
        (
            status = 200,
            description = "Updated english word meaning.",
            body = EnglishWordMeaningUpdatedResponse
        ),
        (
            status = 404,
            response = inline(AsErrorReason<EnglishWordNotFound>)
        ),
        (
            status = 404,
            response = inline(AsErrorReason<EnglishWordMeaningNotFound>)
        ),
        openapi::response::UuidUrlParameterError,
        openapi::response::MissingAuthentication,
        openapi::response::MissingPermissions<requires::WordUpdate, 1>,
        openapi::response::InternalServerError,
    )
)]
#[patch("/{english_word_meaning_id}")]
pub async fn update_english_word_meaning(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(String, String)>,
    request_data: web::Json<EnglishWordMeaningUpdateRequest>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;
    let mut transaction = database_connection.transaction().begin().await?;

    require_user_authentication_and_permissions!(
        &mut transaction,
        authentication,
        Permission::WordUpdate
    );


    let target_english_word_id = parse_uuid::<EnglishWordId>(&parameters.0)?;
    let target_english_word_meaning_id = parse_uuid::<EnglishWordMeaningId>(&parameters.1)?;

    let new_word_meaning_data = request_data.into_inner();



    let word_exists =
        entities::EnglishWordQuery::exists_by_id(&mut transaction, target_english_word_id).await?;

    if !word_exists {
        return EndpointResponseBuilder::not_found()
            .with_error_reason(WordErrorReason::word_not_found())
            .build();
    }


    let word_and_meaning_id_pair_exists =
        entities::EnglishWordMeaningQuery::exists_by_meaning_and_word_id(
            &mut transaction,
            target_english_word_id,
            target_english_word_meaning_id,
        )
        .await?;

    if !word_and_meaning_id_pair_exists {
        return EndpointResponseBuilder::not_found()
            .with_error_reason(WordErrorReason::word_meaning_not_found())
            .build();
    }


    let updated_meaning_successfully = entities::EnglishWordMeaningMutation::update(
        &mut transaction,
        target_english_word_meaning_id,
        EnglishWordMeaningUpdate {
            disambiguation: new_word_meaning_data.disambiguation,
            abbreviation: new_word_meaning_data.abbreviation,
            description: new_word_meaning_data.description,
        },
    )
    .await?;


    // When zero rows are affected by the query, that means there is no such english word meaning,
    // but because we run this in a repeatable read transaction, this should be impossible.
    if !updated_meaning_successfully {
        return Err(EndpointError::invalid_database_state(
            "english word mutation should have updated at least one row",
        ));
    }



    let updated_meaning = entities::EnglishWordMeaningQuery::get(
        &mut transaction,
        target_english_word_id,
        target_english_word_meaning_id,
    )
    .await?;

    let Some(updated_meaning) = updated_meaning else {
        return Err(EndpointError::invalid_database_state(
            "after updating an english word meaning, we could not fetch \
             the full (exact same) meaning inside the transaction",
        ));
    };


    transaction.commit().await?;


    EndpointResponseBuilder::ok()
        .with_json_body(EnglishWordMeaningUpdatedResponse {
            meaning: updated_meaning.into_api_model(),
        })
        .build()
}




/// Delete an english word meaning
///
/// This endpoint deletes an english word meaning.
///
///
/// # Authentication & Required permissions
/// - Authentication **is** required.
/// - The caller must have the `word:update` permission.
#[utoipa::path(
    delete,
    path = "/dictionary/english/{english_word_id}/meaning/{english_word_meaning_id}",
    tag = "dictionary:english:meaning",
    params(
        (
            "english_word_id" = String,
            Path,
            format = Uuid,
            description = "UUID of the english word related to the meaning."
        ),
        (
            "english_word_meaning_id" = String,
            Path,
            format = Uuid,
            description = "UUID of the english word meaning to modify."
        )
    ),
    responses(
        (
            status = 200,
            description = "The requested english word meaning has been deleted.",
        ),
        (
            status = 404,
            response = inline(AsErrorReason<EnglishWordNotFound>)
        ),
        (
            status = 404,
            response = inline(AsErrorReason<EnglishWordMeaningNotFound>)
        ),
        openapi::response::UuidUrlParameterError,
        openapi::response::MissingAuthentication,
        openapi::response::MissingPermissions<requires::WordUpdate, 1>,
        openapi::response::InternalServerError,
    )
)]
#[delete("/{english_word_meaning_id}")]
pub async fn delete_english_word_meaning(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(String, String)>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;
    let mut transaction = database_connection.transaction().begin().await?;

    require_user_authentication_and_permissions!(
        &mut transaction,
        authentication,
        Permission::WordUpdate
    );


    let target_english_word_id = parse_uuid::<EnglishWordId>(&parameters.0)?;
    let target_english_word_meaning_id = parse_uuid::<EnglishWordMeaningId>(&parameters.1)?;


    let word_exists =
        entities::EnglishWordQuery::exists_by_id(&mut transaction, target_english_word_id).await?;

    if !word_exists {
        return EndpointResponseBuilder::not_found()
            .with_error_reason(WordErrorReason::word_not_found())
            .build();
    }


    let word_to_meaning_relationship_exists =
        entities::EnglishWordMeaningQuery::exists_by_meaning_and_word_id(
            &mut transaction,
            target_english_word_id,
            target_english_word_meaning_id,
        )
        .await?;

    if !word_to_meaning_relationship_exists {
        return EndpointResponseBuilder::not_found()
            .with_error_reason(WordErrorReason::word_meaning_not_found())
            .build();
    }


    let successfully_deleted_meaning = entities::EnglishWordMeaningMutation::delete(
        &mut transaction,
        target_english_word_meaning_id,
    )
    .await?;


    if !successfully_deleted_meaning {
        return Err(EndpointError::invalid_database_state(
            "after checking that the english word meaning exists, \
             we could not delete that very same meaning inside the same transaction",
        ));
    }


    transaction.commit().await?;


    Ok(HttpResponse::Ok().finish())
}




pub fn english_word_meaning_router() -> Scope {
    web::scope("/{english_word_id}/meaning")
        .service(get_all_english_word_meanings)
        .service(create_english_word_meaning)
        .service(update_english_word_meaning)
        .service(delete_english_word_meaning)
}
