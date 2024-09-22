use actix_web::{delete, get, patch, post, web, HttpResponse, Scope};
use chrono::{DateTime, Utc};
use kolomoni_auth::Permission;
use kolomoni_core::id::{CategoryId, EnglishWordId, EnglishWordMeaningId};
use kolomoni_database::entities::{self, EnglishWordMeaningUpdate, NewEnglishWordMeaning};
use serde::{Deserialize, Serialize};
use sqlx::{types::Uuid, Acquire};
use utoipa::ToSchema;

use crate::{
    api::{
        errors::{APIError, EndpointResult},
        macros::ContextlessResponder,
        traits::IntoApiModel,
        v1::dictionary::slovene::meaning::ShallowSloveneWordMeaning,
    },
    authentication::UserAuthenticationExtractor,
    impl_json_response_builder,
    obtain_database_connection,
    require_permission,
    require_permission_with_optional_authentication,
    state::ApplicationState,
};


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct ShallowEnglishWordMeaning {
    pub meaning_id: EnglishWordMeaningId,

    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,

    pub categories: Vec<CategoryId>,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct EnglishWordMeaning {
    pub meaning_id: EnglishWordMeaningId,

    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,
}

impl IntoApiModel for entities::EnglishWordMeaningModel {
    type ApiModel = EnglishWordMeaning;

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


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct EnglishWordMeaningWithCategoriesAndTranslations {
    pub meaning_id: EnglishWordMeaningId,

    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,

    pub categories: Vec<CategoryId>,

    pub translates_into: Vec<ShallowSloveneWordMeaning>,
}


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct EnglishWordMeaningsResponse {
    pub meanings: Vec<EnglishWordMeaningWithCategoriesAndTranslations>,
}

impl_json_response_builder!(EnglishWordMeaningsResponse);



#[get("")]
pub async fn get_all_english_word_meanings(
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


    let target_english_word_id = EnglishWordId::new(parameters.into_inner().0);


    let english_word_meanings = entities::EnglishWordMeaningQuery::get_all_by_english_word_id(
        &mut database_connection,
        target_english_word_id,
    )
    .await?;


    Ok(EnglishWordMeaningsResponse {
        meanings: english_word_meanings
            .into_iter()
            .map(|meaning| meaning.into_api_model())
            .collect(),
    }
    .into_response())
}


// TODO could be nice to submit initial categories with this as well?
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct NewEnglishWordMeaningRequest {
    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,
}


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct NewEnglishWordMeaningCreatedResponse {
    pub meaning: EnglishWordMeaning,
}

impl_json_response_builder!(NewEnglishWordMeaningCreatedResponse);



#[post("")]
pub async fn create_english_word_meaning(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(Uuid,)>,
    request_data: web::Json<NewEnglishWordMeaningRequest>,
) -> EndpointResult {
    let mut database_connection = obtain_database_connection!(state);
    let mut transaction = database_connection.begin().await?;

    require_permission!(
        &mut transaction,
        authentication,
        Permission::WordUpdate
    );


    let target_english_word_id = EnglishWordId::new(parameters.into_inner().0);
    let new_word_meaning_data = request_data.into_inner();


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


    Ok(NewEnglishWordMeaningCreatedResponse {
        meaning: newly_created_meaning.into_api_model(),
    }
    .into_response())
}


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct EnglishWordMeaningUpdateRequest {
    #[serde(default, with = "::serde_with::rust::double_option")]
    pub disambiguation: Option<Option<String>>,

    #[serde(default, with = "::serde_with::rust::double_option")]
    pub abbreviation: Option<Option<String>>,

    #[serde(default, with = "::serde_with::rust::double_option")]
    pub description: Option<Option<String>>,
}


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct EnglishWordMeaningUpdatedResponse {
    pub meaning: EnglishWordMeaningWithCategoriesAndTranslations,
}

impl_json_response_builder!(EnglishWordMeaningUpdatedResponse);


#[patch("/{english_word_meaning_id}")]
pub async fn update_english_word_meaning(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(Uuid, Uuid)>,
    request_data: web::Json<EnglishWordMeaningUpdateRequest>,
) -> EndpointResult {
    let mut database_connection = obtain_database_connection!(state);
    let mut transaction = database_connection.begin().await?;

    require_permission!(
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
        return Err(APIError::not_found());
    }



    let updated_meaning = entities::EnglishWordMeaningQuery::get(
        &mut transaction,
        target_english_word_id,
        target_english_word_meaning_id,
    )
    .await?;

    let Some(updated_meaning) = updated_meaning else {
        return Err(APIError::internal_error_with_reason(
            "database inconsistency: after updating an english word meaning \
            we could not fetch the exact same meaning",
        ));
    };


    transaction.commit().await?;


    Ok(EnglishWordMeaningUpdatedResponse {
        meaning: updated_meaning.into_api_model(),
    }
    .into_response())
}



#[delete("/{english_word_meaning_id}")]
pub async fn delete_english_word_meaning(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(Uuid, Uuid)>,
) -> EndpointResult {
    let mut database_connection = obtain_database_connection!(state);
    let mut transaction = database_connection.begin().await?;

    require_permission!(
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


    let successfully_deleted_meaning = entities::EnglishWordMeaningMutation::delete(
        &mut transaction,
        target_english_word_meaning_id,
    )
    .await?;


    if !successfully_deleted_meaning {
        return Err(APIError::not_found());
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
