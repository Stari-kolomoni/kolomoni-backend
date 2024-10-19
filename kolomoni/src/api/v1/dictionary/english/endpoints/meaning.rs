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
use kolomoni_database::entities::{self, EnglishWordMeaningUpdate, NewEnglishWordMeaning};
use sqlx::types::Uuid;

use crate::api::openapi::response::AsErrorReason;
use crate::api::v1::dictionary::english::EnglishWordNotFound;
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
/// - Authentication is not required.
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
        )
    )
)]
#[get("")]
pub async fn get_all_english_word_meanings(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(Uuid,)>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;

    require_permission_with_optional_authentication!(
        &mut database_connection,
        authentication,
        Permission::WordRead
    );


    let target_english_word_id = EnglishWordId::new(parameters.into_inner().0);


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




#[post("")]
pub async fn create_english_word_meaning(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(Uuid,)>,
    request_data: web::Json<NewEnglishWordMeaningRequest>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;
    let mut transaction = database_connection.transaction().begin().await?;

    require_user_authentication_and_permissions!(
        &mut transaction,
        authentication,
        Permission::WordUpdate
    );


    let target_english_word_id = EnglishWordId::new(parameters.into_inner().0);
    let new_word_meaning_data = request_data.into_inner();


    // TODO need to check for duplicate meanings (+ in slovene as well)

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




#[patch("/{english_word_meaning_id}")]
pub async fn update_english_word_meaning(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(Uuid, Uuid)>,
    request_data: web::Json<EnglishWordMeaningUpdateRequest>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;
    let mut transaction = database_connection.transaction().begin().await?;

    require_user_authentication_and_permissions!(
        &mut transaction,
        authentication,
        Permission::WordUpdate
    );


    let (target_english_word_id, target_english_word_meaning_id) = {
        let url_parameters = parameters.into_inner();

        let target_english_word_id = EnglishWordId::new(url_parameters.0);
        let target_english_word_meaning_id = EnglishWordMeaningId::new(url_parameters.1);

        (
            target_english_word_id,
            target_english_word_meaning_id,
        )
    };

    let new_word_meaning_data = request_data.into_inner();

    // TODO we don't verify english word ID validity here, is that okay?


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


    // When zero rows are affected by the query, that means there is no such english word meaning.
    if !updated_meaning_successfully {
        return EndpointResponseBuilder::not_found()
            .with_error_reason(WordErrorReason::word_meaning_not_found())
            .build();
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




#[delete("/{english_word_meaning_id}")]
pub async fn delete_english_word_meaning(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(Uuid, Uuid)>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;
    let mut transaction = database_connection.transaction().begin().await?;

    require_user_authentication_and_permissions!(
        &mut transaction,
        authentication,
        Permission::WordUpdate
    );


    let (target_english_word_id, target_english_word_meaning_id) = {
        let url_parameters = parameters.into_inner();

        let target_english_word_id = EnglishWordId::new(url_parameters.0);
        let target_english_word_meaning_id = EnglishWordMeaningId::new(url_parameters.1);

        (
            target_english_word_id,
            target_english_word_meaning_id,
        )
    };


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
