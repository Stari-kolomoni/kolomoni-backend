use actix_web::{delete, get, patch, post, web, Scope};
use kolomoni_core::api_models::WordErrorReason;
use kolomoni_core::permissions::Permission;
use kolomoni_core::{
    api_models::{
        NewSloveneWordMeaningCreatedResponse,
        NewSloveneWordMeaningRequest,
        SloveneWordMeaningUpdateRequest,
        SloveneWordMeaningUpdatedResponse,
        SloveneWordMeaningsResponse,
    },
    ids::{SloveneWordId, SloveneWordMeaningId},
};
use kolomoni_database::entities::{
    self,
    NewSloveneWordMeaning,
    SloveneWordMeaningLookup,
    SloveneWordMeaningUpdate,
};

use crate::api::openapi;
use crate::api::openapi::response::{requires, AsErrorReason};
use crate::api::v1::dictionary::slovene::SloveneWordNotFound;
use crate::declare_openapi_error_reason_response;
use crate::{
    api::{
        errors::{EndpointError, EndpointResponseBuilder, EndpointResult},
        traits::IntoApiModel,
        v1::dictionary::parse_uuid,
    },
    authentication::UserAuthenticationExtractor,
    require_permission_with_optional_authentication,
    require_user_authentication_and_permissions,
    state::ApplicationState,
};




/// Get all word meanings for a given slovene word
///
/// This endpoint returns a list of all word meanings
/// that the specified slovene word has.
///
///
/// # Authentication & Required permissions
/// - Authentication **is not** required.
/// - The caller must have the `word:read` permission, which is currently
///   blanket-granted to both unauthenticated and authenticated users.
#[utoipa::path(
    get,
    path = "/dictionary/slovene/{slovene_word_id}/meaning",
    tag = "dictionary:slovene:meaning",
    params(
        (
            "slovene_word_id" = String,
            Path,
            format = Uuid,
            description = "UUID of the slovene word to get meanings for."
        )
    ),
    responses(
        (
            status = 200,
            description = "Requested slovene word meanings.",
            body = SloveneWordMeaningsResponse,
        ),
        (
            status = 404,
            response = inline(AsErrorReason<SloveneWordNotFound>)
        ),
        openapi::response::UuidUrlParameterError,
        openapi::response::MissingPermissions<requires::WordRead, 1>,
        openapi::response::InternalServerError,
    )
)]
#[get("")]
pub async fn get_all_slovene_word_meanings(
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


    let target_slovene_word_id = parse_uuid::<SloveneWordId>(parameters.into_inner().0)?;


    let slovene_word_exists =
        entities::SloveneWordQuery::exists_by_id(&mut database_connection, target_slovene_word_id)
            .await?;

    if !slovene_word_exists {
        return EndpointResponseBuilder::not_found()
            .with_error_reason(WordErrorReason::word_not_found())
            .build();
    }


    let slovene_word_meanings = entities::SloveneWordMeaningQuery::get_all_by_slovene_word_id(
        &mut database_connection,
        target_slovene_word_id,
    )
    .await?;


    EndpointResponseBuilder::ok()
        .with_json_body(SloveneWordMeaningsResponse {
            meanings: slovene_word_meanings
                .into_iter()
                .map(|meaning| meaning.into_api_model())
                .collect(),
        })
        .build()
}



declare_openapi_error_reason_response!(
    pub struct SloveneWordMeaningAlreadyExists {
        description => "A slovene word meaning with the given fields already exists.",
        reason => WordErrorReason::identical_word_meaning_already_exists()
    }
);


/// Create a new slovene word meaning
///
/// This endpoint creates a new slovene word meaning with
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
    path = "/dictionary/slovene/{slovene_word_id}/meaning",
    tag = "dictionary:slovene:meaning",
    params(
        (
            "slovene_word_id" = String,
            Path,
            format = Uuid,
            description = "UUID of the slovene word to associate the meaning with."
        )
    ),
    request_body(
        content = NewSloveneWordMeaningRequest
    ),
    responses(
        (
            status = 200,
            description = "Newly-created slovene word meaning.",
            body = NewSloveneWordMeaningCreatedResponse
        ),
        (
            status = 409,
            response = inline(AsErrorReason<SloveneWordMeaningAlreadyExists>)
        ),
        openapi::response::UuidUrlParameterError,
        openapi::response::MissingAuthentication,
        openapi::response::MissingPermissions<requires::WordUpdate, 1>,
        openapi::response::InternalServerError,
    )
)]
#[post("")]
pub async fn create_slovene_word_meaning(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(String,)>,
    request_data: web::Json<NewSloveneWordMeaningRequest>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;
    let mut transaction = database_connection.transaction().begin().await?;

    require_user_authentication_and_permissions!(
        &mut transaction,
        authentication,
        Permission::WordUpdate
    );


    let target_slovene_word_id = parse_uuid::<SloveneWordId>(parameters.into_inner().0)?;

    let new_word_meaning_data = request_data.into_inner();


    let identical_meaning_already_exists =
        entities::SloveneWordMeaningQuery::exists_by_distinguishing_fields(
            &mut transaction,
            SloveneWordMeaningLookup {
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



    let newly_created_meaning = entities::SloveneWordMeaningMutation::create(
        &mut transaction,
        target_slovene_word_id,
        NewSloveneWordMeaning {
            abbreviation: new_word_meaning_data.abbreviation,
            description: new_word_meaning_data.description,
            disambiguation: new_word_meaning_data.disambiguation,
        },
    )
    .await?;


    transaction.commit().await?;


    EndpointResponseBuilder::ok()
        .with_json_body(NewSloveneWordMeaningCreatedResponse {
            meaning: newly_created_meaning.into_api_model(),
        })
        .build()
}



declare_openapi_error_reason_response!(
    pub struct SloveneWordMeaningNotFound {
        description => "The slovene word exists, but its associated word meaning does not.",
        reason => WordErrorReason::word_meaning_not_found()
    }
);



/// Modifies a slovene word meaning
///
/// This endpoint modifies a slovene word meaning.
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
    path = "/dictionary/slovene/{slovene_word_id}/meaning/{slovene_word_meaning_id}",
    tag = "dictionary:slovene:meaning",
    params(
        (
            "slovene_word_id" = String,
            Path,
            format = Uuid,
            description = "UUID of the slovene word to related to the meaning."
        ),
        (
            "slovene_word_meaning_id" = String,
            Path,
            format = Uuid,
            description = "UUID of the slovene word meaning to modify."
        )
    ),
    request_body(
        content = SloveneWordMeaningUpdateRequest
    ),
    responses(
        (
            status = 200,
            description = "Updated slovene word meaning.",
            body = SloveneWordMeaningUpdatedResponse
        ),
        (
            status = 404,
            response = inline(AsErrorReason<SloveneWordNotFound>)
        ),
        (
            status = 404,
            response = inline(AsErrorReason<SloveneWordMeaningNotFound>)
        ),
        openapi::response::UuidUrlParameterError,
        openapi::response::MissingAuthentication,
        openapi::response::MissingPermissions<requires::WordUpdate, 1>,
        openapi::response::InternalServerError,
    )
)]
#[patch("/{slovene_word_meaning_id}")]
pub async fn update_slovene_word_meaning(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(String, String)>,
    request_data: web::Json<SloveneWordMeaningUpdateRequest>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;
    let mut transaction = database_connection.transaction().begin().await?;

    require_user_authentication_and_permissions!(
        &mut transaction,
        authentication,
        Permission::WordUpdate
    );



    let url_parameters = parameters.into_inner();

    let target_slovene_word_id = parse_uuid::<SloveneWordId>(url_parameters.0)?;
    let target_slovene_word_meaning_id = parse_uuid::<SloveneWordMeaningId>(url_parameters.1)?;


    let new_word_meaning_data = request_data.into_inner();



    let word_exists =
        entities::SloveneWordQuery::exists_by_id(&mut transaction, target_slovene_word_id).await?;

    if !word_exists {
        return EndpointResponseBuilder::not_found()
            .with_error_reason(WordErrorReason::word_not_found())
            .build();
    }


    let word_and_meaning_id_pair_exists =
        entities::SloveneWordMeaningQuery::exists_by_meaning_and_word_id(
            &mut transaction,
            target_slovene_word_id,
            target_slovene_word_meaning_id,
        )
        .await?;

    if !word_and_meaning_id_pair_exists {
        return EndpointResponseBuilder::not_found()
            .with_error_reason(WordErrorReason::word_meaning_not_found())
            .build();
    }


    let successfully_updated_meaning = entities::SloveneWordMeaningMutation::update(
        &mut transaction,
        target_slovene_word_meaning_id,
        SloveneWordMeaningUpdate {
            disambiguation: new_word_meaning_data.disambiguation,
            abbreviation: new_word_meaning_data.abbreviation,
            description: new_word_meaning_data.description,
        },
    )
    .await?;


    // When zero rows are affected by the query, that means there is no such slovene word meaning.
    if !successfully_updated_meaning {
        return EndpointResponseBuilder::not_found()
            .with_error_reason(WordErrorReason::word_meaning_not_found())
            .build();
    }



    let updated_meaning = entities::SloveneWordMeaningQuery::get(
        &mut transaction,
        target_slovene_word_id,
        target_slovene_word_meaning_id,
    )
    .await?;

    let Some(updated_meaning) = updated_meaning else {
        return Err(EndpointError::invalid_database_state(
            "after having just updated a slovene word meaning, we could not fetch \
             the full (exact same) meaning inside the same transaction",
        ));
    };

    transaction.commit().await?;


    EndpointResponseBuilder::ok()
        .with_json_body(SloveneWordMeaningUpdatedResponse {
            meaning: updated_meaning.into_api_model(),
        })
        .build()
}




/// Delete a slovene word meaning
///
/// This endpoint deletes a slovene word meaning.
///
///
/// # Authentication & Required permissions
/// - Authentication **is** required.
/// - The caller must have the `word:update` permission.
#[utoipa::path(
    delete,
    path = "/dictionary/slovene/{slovene_word_id}/meaning/{slovene_word_meaning_id}",
    tag = "dictionary:slovene:meaning",
    params(
        (
            "slovene_word_id" = String,
            Path,
            format = Uuid,
            description = "UUID of the slovene word to related to the meaning."
        ),
        (
            "slovene_word_meaning_id" = String,
            Path,
            format = Uuid,
            description = "UUID of the slovene word meaning to modify."
        )
    ),
    responses(
        (
            status = 200,
            description = "The requested slovene word meaning has been deleted."
        ),
        (
            status = 404,
            response = inline(AsErrorReason<SloveneWordNotFound>)
        ),
        (
            status = 404,
            response = inline(AsErrorReason<SloveneWordMeaningNotFound>)
        ),
        openapi::response::UuidUrlParameterError,
        openapi::response::MissingAuthentication,
        openapi::response::MissingPermissions<requires::WordUpdate, 1>,
        openapi::response::InternalServerError,
    )
)]
#[delete("/{slovene_word_meaning_id}")]
pub async fn delete_slovene_word_meaning(
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


    let url_parameters = parameters.into_inner();

    let target_slovene_word_id = parse_uuid::<SloveneWordId>(url_parameters.0)?;
    let target_slovene_word_meaning_id = parse_uuid::<SloveneWordMeaningId>(url_parameters.1)?;


    let word_to_meaning_relationship_exists =
        entities::SloveneWordMeaningQuery::exists_by_meaning_and_word_id(
            &mut transaction,
            target_slovene_word_id,
            target_slovene_word_meaning_id,
        )
        .await?;

    if !word_to_meaning_relationship_exists {
        return EndpointResponseBuilder::not_found()
            .with_error_reason(WordErrorReason::word_meaning_not_found())
            .build();
    }


    let successfully_deleted_meaning = entities::SloveneWordMeaningMutation::delete(
        &mut transaction,
        target_slovene_word_meaning_id,
    )
    .await?;

    if !successfully_deleted_meaning {
        return Err(EndpointError::invalid_database_state(
            "after having just checked that a slovene word meaning exists, we could not \
             delete it inside the same transaction",
        ));
    }


    transaction.commit().await?;


    EndpointResponseBuilder::ok().build()
}

// TODO next up: refactor names and structure, then look at aligning the utoipa docs with the actual endpoints again



pub fn slovene_word_meaning_router() -> Scope {
    web::scope("/{slovene_word_id}/meaning")
        .service(get_all_slovene_word_meanings)
        .service(create_slovene_word_meaning)
        .service(update_slovene_word_meaning)
        .service(delete_slovene_word_meaning)
}
