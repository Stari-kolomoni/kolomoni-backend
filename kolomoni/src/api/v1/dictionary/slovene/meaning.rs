use actix_web::{delete, get, patch, post, web, HttpResponse, Scope};
use chrono::{DateTime, Utc};
use kolomoni_auth::Permission;
use kolomoni_core::{
    api_models::Category,
    id::{CategoryId, SloveneWordId, SloveneWordMeaningId},
};
use kolomoni_database::entities::{self, NewSloveneWordMeaning, SloveneWordMeaningUpdate};
use serde::{Deserialize, Serialize};
use sqlx::Acquire;
use utoipa::ToSchema;
use uuid::Uuid;

use super::word::SloveneWordMeaningWithCategoriesAndTranslations;
use crate::{
    api::{
        errors::{APIError, EndpointResult},
        macros::ContextlessResponder,
        traits::IntoApiModel,
        v1::dictionary::english::meaning::ShallowEnglishWordMeaning,
    },
    authentication::UserAuthenticationExtractor,
    impl_json_response_builder,
    obtain_database_connection,
    require_permission_OLD,
    require_permission_with_optional_authentication,
    state::ApplicationState,
};


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct ShallowSloveneWordMeaning {
    pub meaning_id: SloveneWordMeaningId,

    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,

    pub categories: Vec<CategoryId>,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,
}

// TODO refactor these names, this one is the same as ShallowSloveneWordMeaning, but without categories
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct SloveneWordMeaning {
    pub meaning_id: SloveneWordMeaningId,

    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,
}

impl IntoApiModel for entities::SloveneWordMeaningModel {
    type ApiModel = SloveneWordMeaning;

    fn into_api_model(self) -> Self::ApiModel {
        Self::ApiModel {
            meaning_id: self.id,
            disambiguation: self.disambiguation,
            abbreviation: self.abbreviation,
            description: self.description,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
        }
    }
}


/*
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct SloveneWordMeaningWithCategoriesAndTranslations {
    pub meaning_id: SloveneWordMeaningId,

    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,

    pub categories: Vec<CategoryId>,

    pub translates_into: Vec<ShallowEnglishWordMeaning>,
} */


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct SloveneWordMeaningsResponse {
    pub meanings: Vec<SloveneWordMeaningWithCategoriesAndTranslations>,
}

impl_json_response_builder!(SloveneWordMeaningsResponse);



#[get("")]
pub async fn get_all_slovene_word_meanings(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(Uuid,)>,
) -> EndpointResult {
    let mut database_connection = obtain_database_connection!(state);

    require_permission_with_optional_authentication!(
        &mut database_connection,
        authentication,
        Permission::WordRead
    );


    let target_slovene_word_id = SloveneWordId::new(parameters.into_inner().0);


    let slovene_word_meanings = entities::SloveneWordMeaningQuery::get_all_by_slovene_word_id(
        &mut database_connection,
        target_slovene_word_id,
    )
    .await?;

    Ok(SloveneWordMeaningsResponse {
        meanings: slovene_word_meanings
            .into_iter()
            .map(|meaning| meaning.into_api_model())
            .collect(),
    }
    .into_response())
}



// TODO could be nice to submit initial categories with this as well? (see also english version of this)
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct NewSloveneWordMeaningRequest {
    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,
}


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct NewSloveneWordMeaningCreatedResponse {
    pub meaning: SloveneWordMeaning,
}

impl_json_response_builder!(NewSloveneWordMeaningCreatedResponse);



#[post("")]
pub async fn create_slovene_word_meaning(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(Uuid,)>,
    request_data: web::Json<NewSloveneWordMeaningRequest>,
) -> EndpointResult {
    let mut database_connection = obtain_database_connection!(state);
    let mut transaction = database_connection.begin().await?;

    require_permission_OLD!(
        &mut transaction,
        authentication,
        Permission::WordUpdate
    );


    let target_slovene_word_id = SloveneWordId::new(parameters.into_inner().0);
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


    Ok(NewSloveneWordMeaningCreatedResponse {
        meaning: newly_created_meaning.into_api_model(),
    }
    .into_response())
}


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct SloveneWordMeaningUpdateRequest {
    #[serde(default, with = "::serde_with::rust::double_option")]
    pub disambiguation: Option<Option<String>>,

    #[serde(default, with = "::serde_with::rust::double_option")]
    pub abbreviation: Option<Option<String>>,

    #[serde(default, with = "::serde_with::rust::double_option")]
    pub description: Option<Option<String>>,
}


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct SloveneWordMeaningUpdatedResponse {
    pub meaning: SloveneWordMeaningWithCategoriesAndTranslations,
}

impl_json_response_builder!(SloveneWordMeaningUpdatedResponse);



#[patch("/{slovene_word_meaning_id}")]
pub async fn update_slovene_word_meaning(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(Uuid, Uuid)>,
    request_data: web::Json<SloveneWordMeaningUpdateRequest>,
) -> EndpointResult {
    let mut database_connection = obtain_database_connection!(state);
    let mut transaction = database_connection.begin().await?;

    require_permission_OLD!(
        &mut transaction,
        authentication,
        Permission::WordUpdate
    );



    let (target_slovene_word_id, target_slovene_word_meaning_id) = {
        let url_parameters = parameters.into_inner();

        let target_slovene_word_id = SloveneWordId::new(url_parameters.0);
        let target_slovene_word_meaning_id = SloveneWordMeaningId::new(url_parameters.1);

        (
            target_slovene_word_id,
            target_slovene_word_meaning_id,
        )
    };

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
        return Err(APIError::not_found());
    }



    let updated_meaning = entities::SloveneWordMeaningQuery::get(
        &mut transaction,
        target_slovene_word_id,
        target_slovene_word_meaning_id,
    )
    .await?;

    let Some(updated_meaning) = updated_meaning else {
        return Err(APIError::internal_error_with_reason(
            "database inconsistency: after updating a slovene word meaning \
            we could not fetch the exact same meaning",
        ));
    };

    transaction.commit().await?;


    Ok(SloveneWordMeaningUpdatedResponse {
        meaning: updated_meaning.into_api_model(),
    }
    .into_response())
}



#[delete("/{slovene_word_meaning_id}")]
pub async fn delete_slovene_word_meaning(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(Uuid, Uuid)>,
) -> EndpointResult {
    let mut database_connection = obtain_database_connection!(state);
    let mut transaction = database_connection.begin().await?;

    require_permission_OLD!(
        &mut transaction,
        authentication,
        Permission::WordUpdate
    );


    let (target_slovene_word_id, target_slovene_word_meaning_id) = {
        let url_parameters = parameters.into_inner();

        let target_slovene_word_id = SloveneWordId::new(url_parameters.0);
        let target_slovene_word_meaning_id = SloveneWordMeaningId::new(url_parameters.1);

        (
            target_slovene_word_id,
            target_slovene_word_meaning_id,
        )
    };


    let successfully_deleted_meaning = entities::SloveneWordMeaningMutation::delete(
        &mut transaction,
        target_slovene_word_meaning_id,
    )
    .await?;

    if !successfully_deleted_meaning {
        return Err(APIError::not_found());
    }


    transaction.commit().await?;


    Ok(HttpResponse::Ok().finish())
}

// TODO next up: refactor names and structure, then look at aligning the utoipa docs with the actual endpoints again



pub fn slovene_word_meaning_router() -> Scope {
    web::scope("/{slovene_word_id}/meaning")
        .service(get_all_slovene_word_meanings)
        .service(create_slovene_word_meaning)
        .service(update_slovene_word_meaning)
        .service(delete_slovene_word_meaning)
}
