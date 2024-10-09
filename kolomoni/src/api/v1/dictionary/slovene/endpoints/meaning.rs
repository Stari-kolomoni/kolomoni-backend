use actix_web::{delete, get, patch, post, web, Scope};
use kolomoni_auth::Permission;
use kolomoni_core::{
    api_models::{
        NewSloveneWordMeaningCreatedResponse,
        NewSloveneWordMeaningRequest,
        SloveneWordMeaningUpdateRequest,
        SloveneWordMeaningUpdatedResponse,
        SloveneWordMeaningsResponse,
    },
    id::{SloveneWordId, SloveneWordMeaningId},
};
use kolomoni_database::entities::{self, NewSloveneWordMeaning, SloveneWordMeaningUpdate};
use sqlx::Acquire;

use crate::{
    api::{
        errors::{EndpointError, EndpointResponseBuilder, EndpointResult, WordErrorReason},
        traits::IntoApiModel,
        v1::dictionary::parse_uuid,
    },
    authentication::UserAuthenticationExtractor,
    require_permission_with_optional_authentication,
    require_user_authentication_and_permissions,
    state::ApplicationState,
};




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




#[post("")]
pub async fn create_slovene_word_meaning(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(String,)>,
    request_data: web::Json<NewSloveneWordMeaningRequest>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;
    let mut transaction = database_connection.begin().await?;

    require_user_authentication_and_permissions!(
        &mut transaction,
        authentication,
        Permission::WordUpdate
    );


    let target_slovene_word_id = parse_uuid::<SloveneWordId>(parameters.into_inner().0)?;

    let new_word_meaning_data = request_data.into_inner();



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




#[patch("/{slovene_word_meaning_id}")]
pub async fn update_slovene_word_meaning(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(String, String)>,
    request_data: web::Json<SloveneWordMeaningUpdateRequest>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;
    let mut transaction = database_connection.begin().await?;

    require_user_authentication_and_permissions!(
        &mut transaction,
        authentication,
        Permission::WordUpdate
    );



    let url_parameters = parameters.into_inner();

    let target_slovene_word_id = parse_uuid::<SloveneWordId>(url_parameters.0)?;
    let target_slovene_word_meaning_id = parse_uuid::<SloveneWordMeaningId>(url_parameters.1)?;


    let new_word_meaning_data = request_data.into_inner();

    // TODO we don't verify slovene word ID validity here, is that okay?



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




#[delete("/{slovene_word_meaning_id}")]
pub async fn delete_slovene_word_meaning(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(String, String)>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;
    let mut transaction = database_connection.begin().await?;

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
