use actix_http::StatusCode;
use actix_web::{delete, get, patch, post, web, HttpResponse, Scope};
use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use kolomoni_auth::Permission;
use kolomoni_core::id::{CategoryId, SloveneWordId, SloveneWordMeaningId};
use kolomoni_database::entities::{
    self,
    NewSloveneWord,
    SloveneWordFieldsToUpdate,
    SloveneWordsQueryOptions,
};
use serde::{Deserialize, Serialize};
use sqlx::Acquire;
use tracing::info;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    api::{
        errors::{APIError, EndpointResult},
        macros::ContextlessResponder,
        openapi,
        traits::IntoApiModel,
        v1::dictionary::english::meaning::ShallowEnglishWordMeaning,
    },
    authentication::UserAuthenticationExtractor,
    impl_json_response_builder,
    json_error_response_with_reason,
    obtain_database_connection,
    require_user_authentication,
    require_permission_OLD,
    require_permission_with_optional_authentication,
    state::ApplicationState,
};



#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
#[schema(
    example = json!({
        "id": "018dbe00-266e-7398-abd2-0906df0aa345",
        "lemma": "pustolovec",
        "disambiguation": "lik",
        "description": "Igrani ali neigrani liki, ki se odpravijo na pustolovščino.",
        "created_at": "2023-06-27T20:34:27.217273Z",
        "last_modified_at": "2023-06-27T20:34:27.217273Z"
    })
)]
pub struct SloveneWordWithMeanings {
    /// Internal UUID of the word.
    pub id: SloveneWordId,

    /// An abstract or base form of the word.
    pub lemma: String,

    /// When the word was created.
    pub created_at: DateTime<Utc>,

    /// When the word was last modified.
    ///
    /// TODO In the future, this might include last modification time
    ///      of the linked suggestion and translation relationships.
    pub last_modified_at: DateTime<Utc>,

    pub meanings: Vec<SloveneWordMeaningWithCategoriesAndTranslations>,
}


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
}


impl IntoApiModel for entities::SloveneWordMeaningModelWithCategoriesAndTranslations {
    type ApiModel = SloveneWordMeaningWithCategoriesAndTranslations;

    fn into_api_model(self) -> Self::ApiModel {
        Self::ApiModel {
            meaning_id: self.id,
            disambiguation: self.disambiguation,
            abbreviation: self.abbreviation,
            description: self.description,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
            categories: self.categories,
            translates_into: self
                .translates_into
                .into_iter()
                .map(|translation| translation.into_api_model())
                .collect(),
        }
    }
}


impl IntoApiModel for entities::TranslatesIntoEnglishWordMeaningModel {
    type ApiModel = ShallowEnglishWordMeaning;

    fn into_api_model(self) -> Self::ApiModel {
        Self::ApiModel {
            meaning_id: self.word_meaning_id,
            disambiguation: self.disambiguation,
            abbreviation: self.abbreviation,
            description: self.description,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
            categories: self.categories,
        }
    }
}


/*
impl SloveneWord {
    pub fn new_without_expanded_info(slovene_model: entities::word_slovene::Model) -> Self {
        Self {
            id: slovene_model.word_id.to_string(),
            lemma: slovene_model.lemma,
            disambiguation: slovene_model.disambiguation,
            description: slovene_model.description,
            created_at: slovene_model.created_at.to_utc(),
            last_modified_at: slovene_model.last_modified_at.to_utc(),
            categories: Vec::new(),
        }
    }

    pub fn from_word_and_related_info(
        word_model: entities::word_slovene::Model,
        related_slovene_word_info: RelatedSloveneWordInfo,
    ) -> Self {
        let categories = related_slovene_word_info
            .categories
            .into_iter()
            .map(Category::from_database_model)
            .collect();


        Self {
            id: word_model.word_id.to_string(),
            lemma: word_model.lemma,
            disambiguation: word_model.disambiguation,
            description: word_model.description,
            created_at: word_model.created_at.to_utc(),
            last_modified_at: word_model.last_modified_at.to_utc(),
            categories,
        }
    }

    pub fn from_expanded_word_info(expanded_slovene_word: ExpandedSloveneWordInfo) -> Self {
        let word = expanded_slovene_word.word;

        let categories = expanded_slovene_word
            .categories
            .into_iter()
            .map(Category::from_database_model)
            .collect();


        Self {
            id: word.word_id.to_string(),
            lemma: word.lemma,
            disambiguation: word.disambiguation,
            description: word.description,
            created_at: word.created_at.to_utc(),
            last_modified_at: word.last_modified_at.to_utc(),
            categories,
        }
    }
} */



#[derive(Serialize, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Deserialize))]
pub struct SloveneWordsResponse {
    pub slovene_words: Vec<SloveneWordWithMeanings>,
}

impl_json_response_builder!(SloveneWordsResponse);



#[derive(Deserialize, Clone, PartialEq, Eq, Debug, ToSchema, Default)]
#[cfg_attr(feature = "with_test_facilities", derive(Serialize))]
pub struct SloveneWordFilters {
    pub last_modified_after: Option<DateTime<Utc>>,
}


#[derive(Deserialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Serialize))]
pub struct SloveneWordsListRequest {
    pub filters: Option<SloveneWordFilters>,
}


impl IntoApiModel for entities::SloveneWordWithMeaningsModel {
    type ApiModel = SloveneWordWithMeanings;

    fn into_api_model(self) -> Self::ApiModel {
        Self::ApiModel {
            id: self.word_id,
            lemma: self.lemma,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
            meanings: self
                .meanings
                .into_iter()
                .map(|meaning| meaning.into_api_model())
                .collect(),
        }
    }
}



/// List all slovene words
///
/// This endpoint returns a list of all slovene words.
///
/// # Authentication
/// Authentication is *not required* on this endpoint due to blanket grant of
/// the `word:read` permission to unauthenticated users.
#[utoipa::path(
    get,
    path = "/dictionary/slovene",
    tag = "dictionary:slovene",
    request_body(
        content = Option<SloveneWordsListRequest>
    ),
    responses(
        (
            status = 200,
            description = "A list of all slovene words.",
            body = SloveneWordsResponse,
        ),
        openapi::FailedAuthenticationResponses<openapi::RequiresWordRead>,
        openapi::InternalServerErrorResponse,
    )
)]
#[get("")]
pub async fn get_all_slovene_words(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    request_body: Option<web::Json<SloveneWordsListRequest>>,
) -> EndpointResult {
    let mut database_connection = obtain_database_connection!(state);

    require_permission_with_optional_authentication!(
        &mut database_connection,
        authentication,
        Permission::WordRead
    );
    // TODO continue from here


    let word_query_options = request_body
        .map(|options| {
            options
                .into_inner()
                .filters
                .map(|filter_options| SloveneWordsQueryOptions {
                    only_words_modified_after: filter_options.last_modified_after,
                })
        })
        .flatten()
        .unwrap_or_default();


    // Load words from the database.
    let mut words_with_meanings_stream =
        entities::SloveneWordQuery::get_all_slovene_words_with_meanings(
            &mut database_connection,
            word_query_options,
        )
        .await;


    let mut slovene_words = Vec::new();

    while let Some(word_result) = words_with_meanings_stream.next().await {
        slovene_words.push(word_result?.into_api_model());
    }


    Ok(SloveneWordsResponse { slovene_words }.into_response())
}


#[derive(Deserialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Serialize))]
#[schema(
    example = json!({
        "lemma": "pustolovec"
    })
)]
pub struct SloveneWordCreationRequest {
    pub lemma: String,
}

#[derive(Serialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Deserialize))]
#[schema(
    example = json!({
        "word": {
            "id": "018dbe00-266e-7398-abd2-0906df0aa345",
            "lemma": "pustolovec",
            "disambiguation": "lik",
            "description": "Igrani ali neigrani liki, ki se odpravijo na pustolovščino.",
            "added_at": "2023-06-27T20:34:27.217273Z",
            "last_edited_at": "2023-06-27T20:34:27.217273Z"
        }
    })
)]
pub struct SloveneWordCreationResponse {
    pub word: SloveneWordWithMeanings,
}

impl_json_response_builder!(SloveneWordCreationResponse);


impl IntoApiModel for entities::SloveneWordModel {
    type ApiModel = SloveneWordWithMeanings;

    fn into_api_model(self) -> Self::ApiModel {
        Self::ApiModel {
            id: self.word_id,
            lemma: self.lemma,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
            meanings: vec![],
        }
    }
}



/// Create a slovene word
///
/// This endpoint creates a new slovene word in the dictionary.
///
/// # Authentication
/// This endpoint requires authentication and the `word:create` permission.
#[utoipa::path(
    post,
    path = "/dictionary/slovene",
    tag = "dictionary:slovene",
    request_body(
        content = SloveneWordCreationRequest
    ),
    responses(
        (
            status = 200,
            description = "The newly-created slovene word.",
            body = SloveneWordCreationResponse,
        ),
        (
            status = 409,
            description = "Slovene word with the given lemma already exists.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "A slovene word with the given lemma already exists." })
        ),
        openapi::MissingOrInvalidJsonRequestBodyResponse,
        openapi::FailedAuthenticationResponses<openapi::RequiresWordCreate>,
        openapi::InternalServerErrorResponse,
    ),
    security(
        ("access_token" = [])
    )
)]
#[post("")]
pub async fn create_slovene_word(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    creation_request: web::Json<SloveneWordCreationRequest>,
) -> EndpointResult {
    let mut database_connection = obtain_database_connection!(state);
    let mut transaction = database_connection.begin().await?;

    let authenticated_user = require_user_authentication!(authentication);
    require_permission_OLD!(
        &mut transaction,
        authenticated_user,
        Permission::WordCreate
    );



    let creation_request = creation_request.into_inner();


    let word_lemma_already_exists =
        entities::SloveneWordQuery::exists_by_exact_lemma(&mut transaction, &creation_request.lemma)
            .await?;

    if word_lemma_already_exists {
        return Ok(json_error_response_with_reason!(
            StatusCode::CONFLICT,
            "A slovene word with the given lemma already exists."
        ));
    }


    let newly_created_word = entities::SloveneWordMutation::create(
        &mut transaction,
        NewSloveneWord {
            lemma: creation_request.lemma,
        },
    )
    .await?;

    info!(
        created_by_user = %authenticated_user.user_id(),
        "Created new slovene word: {}", newly_created_word.lemma,
    );

    /* TODO pending rewrite of cache layer
    // Signals to the the search indexer that the word has been created.
    state
        .search
        .signal_slovene_word_created_or_updated(newly_created_word.word_id)
        .await
        .map_err(APIError::InternalGenericError)?; */


    Ok(SloveneWordCreationResponse {
        // Newly created words do not belong to any categories.
        word: newly_created_word.into_api_model(),
    }
    .into_response())
}




#[derive(Serialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Deserialize))]
pub struct SloveneWordInfoResponse {
    pub word: SloveneWordWithMeanings,
}

impl_json_response_builder!(SloveneWordInfoResponse);



/// Get a slovene word
///
/// This endpoint returns information about a single slovene word from the dictionary.
///
/// # Authentication
/// Authentication is *not required* on this endpoint due to a blanket grant of
/// the `word:read` permission to unauthenticated users.
#[utoipa::path(
    get,
    path = "/dictionary/slovene/{word_uuid}",
    tag = "dictionary:slovene",
    params(
        (
            "word_uuid" = String,
            Path,
            description = "UUID of the slovene word."
        )
    ),
    responses(
        (
            status = 200,
            description = "Information about the requested slovene word.",
            body = SloveneWordInfoResponse,
        ),
        (
            status = 400,
            description = "Invalid word UUID provided.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "Client error: invalid UUID." })
        ),
        (
            status = 404,
            description = "The requested slovene word does not exist."
        ),
        openapi::FailedAuthenticationResponses<openapi::RequiresWordRead>,
        openapi::InternalServerErrorResponse,
    )
)]
#[get("/{word_uuid}")]
pub async fn get_specific_slovene_word(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(Uuid,)>,
) -> EndpointResult {
    // TODO continue from here
    let mut database_connection = obtain_database_connection!(state);

    require_permission_with_optional_authentication!(
        &mut database_connection,
        authentication,
        Permission::WordRead
    );


    let target_word_uuid = SloveneWordId::new(parameters.into_inner().0);


    let potential_slovene_word = entities::SloveneWordQuery::get_by_id_with_meanings(
        &mut database_connection,
        target_word_uuid,
    )
    .await?;

    let Some(slovene_word_with_meanings) = potential_slovene_word else {
        return Err(APIError::not_found());
    };


    Ok(SloveneWordInfoResponse {
        word: slovene_word_with_meanings.into_api_model(),
    }
    .into_response())
}



/// Fina a slovene word by lemma
///
/// This endpoint returns information about a single slovene word from the dictionary,
/// but takes a lemma as a parameter instead of the word ID.
///
/// Note that this is *not* intended as a search endpoint!
///
/// # Authentication
/// Authentication is *not required* on this endpoint due to a blanket grant of
/// the `word:read` permission to unauthenticated users.
#[utoipa::path(
    get,
    path = "/dictionary/slovene/by-lemma/{word_lemma}",
    tag = "dictionary:slovene",
    params(
        (
            "word_lemma" = String,
            Path,
            description = "Slovene word lemma to look up."
        )
    ),
    responses(
        (
            status = 200,
            description = "Information about the requested slovene word.",
            body = SloveneWordInfoResponse,
        ),
        (
            status = 404,
            description = "The requested slovene word does not exist."
        ),
        openapi::FailedAuthenticationResponses<openapi::RequiresWordRead>,
        openapi::InternalServerErrorResponse,
    )
)]
#[get("/by-lemma/{word_lemma}")]
pub async fn get_specific_slovene_word_by_lemma(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(String,)>,
) -> EndpointResult {
    let mut database_connection = obtain_database_connection!(state);

    require_permission_with_optional_authentication!(
        &mut database_connection,
        authentication,
        Permission::WordRead
    );


    let target_word_lemma = &parameters.into_inner().0;


    let potential_slovene_word = entities::SloveneWordQuery::get_by_exact_lemma_with_meanings(
        &mut database_connection,
        &target_word_lemma,
    )
    .await?;

    let Some(slovene_word_with_meanings) = potential_slovene_word else {
        return Err(APIError::not_found());
    };


    Ok(SloveneWordInfoResponse {
        word: slovene_word_with_meanings.into_api_model(),
    }
    .into_response())
}




#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug, ToSchema, Default)]
pub struct SloveneWordUpdateRequest {
    pub lemma: Option<String>,
}

impl_json_response_builder!(SloveneWordUpdateRequest);



/// Update a slovene word
///
/// This endpoint updates an existing slovene word in the dictionary.
///
/// # Authentication
/// This endpoint requires authentication and the `word:update` permission.
#[utoipa::path(
    patch,
    path = "/dictionary/slovene/{word_uuid}",
    tag = "dictionary:slovene",
    params(
        (
            "word_uuid" = String,
            Path,
            description = "UUID of the slovene word."
        )
    ),
    request_body(
        content = SloveneWordUpdateRequest,
    ),
    responses(
        (
            status = 200,
            description = "Updated slovene word.",
            body = SloveneWordInfoResponse,
        ),
        (
            status = 400,
            description = "Invalid word UUID provided.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "Client error: invalid UUID." })
        ),
        (
            status = 404,
            description = "The requested slovene word does not exist."
        ),
        openapi::MissingOrInvalidJsonRequestBodyResponse,
        openapi::FailedAuthenticationResponses<openapi::RequiresWordUpdate>,
        openapi::InternalServerErrorResponse,
    ),
    security(
        ("access_token" = [])
    )
)]
#[patch("/{word_uuid}")]
pub async fn update_specific_slovene_word(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(Uuid,)>,
    request_data: web::Json<SloveneWordUpdateRequest>,
) -> EndpointResult {
    let mut database_connection = obtain_database_connection!(state);
    let mut transaction = database_connection.begin().await?;

    let authenticated_user = require_user_authentication!(authentication);
    require_permission_OLD!(
        &mut transaction,
        authenticated_user,
        Permission::WordUpdate
    );


    let target_word_id = SloveneWordId::new(parameters.into_inner().0);
    let request_data = request_data.into_inner();



    let target_word_exists =
        entities::SloveneWordQuery::exists_by_id(&mut transaction, target_word_id).await?;

    if !target_word_exists {
        return Err(APIError::not_found());
    }


    let updated_successfully = entities::SloveneWordMutation::update(
        &mut transaction,
        target_word_id,
        SloveneWordFieldsToUpdate {
            new_lemma: request_data.lemma,
        },
    )
    .await?;

    if !updated_successfully {
        transaction.rollback().await?;

        return Err(APIError::internal_error_with_reason(
            "database inconsistency: failed to update slovene word, even though it \
            previously existed inside the same transaction",
        ));
    }


    let updated_word =
        entities::SloveneWordQuery::get_by_id_with_meanings(&mut transaction, target_word_id)
            .await?
            .ok_or_else(|| {
                APIError::internal_error_with_reason(
                    "database inconsistency: word did not exist after updating it \
                    inside the same transaction",
                )
            })?;


    // TODO at the end, go over all endpoints and make sure that they all commit transactions if they use them
    transaction.commit().await?;


    /* TODO pending cache layer rewrite
    // Signals to the the search indexer that the word has been updated.
    state
        .search
        .signal_slovene_word_created_or_updated(updated_word.word_id)
        .await
        .map_err(APIError::InternalGenericError)?; */


    Ok(SloveneWordInfoResponse {
        word: updated_word.into_api_model(),
    }
    .into_response())
}



/// Delete a slovene word
///
/// This endpoint deletes a slovene word from the dictionary.
///
/// # Authentication
/// This endpoint requires authentication and the `word:delete` permission.
#[utoipa::path(
    delete,
    path = "/dictionary/slovene/{word_uuid}",
    tag = "dictionary:slovene",
    params(
        (
            "word_uuid" = String,
            Path,
            description = "UUID of the slovene word to delete."
        )
    ),
    responses(
        (
            status = 200,
            description = "Slovene word deleted.",
        ),
        (
            status = 400,
            description = "Invalid word UUID provided.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "Client error: invalid UUID." })
        ),
        (
            status = 404,
            description = "The given slovene word does not exist."
        ),
        openapi::FailedAuthenticationResponses<openapi::RequiresWordDelete>,
        openapi::InternalServerErrorResponse,
    ),
    security(
        ("access_token" = [])
    )
)]
#[delete("/{word_uuid}")]
pub async fn delete_specific_slovene_word(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(Uuid,)>,
) -> EndpointResult {
    let mut database_connection = obtain_database_connection!(state);
    let mut transaction = database_connection.begin().await?;

    let authenticated_user = require_user_authentication!(authentication);
    require_permission_OLD!(
        &mut transaction,
        authenticated_user,
        Permission::WordDelete
    );


    let target_word_id = SloveneWordId::new(parameters.into_inner().0);


    let target_word_exists =
        entities::SloveneWordQuery::exists_by_id(&mut transaction, target_word_id).await?;

    if !target_word_exists {
        return Err(APIError::not_found());
    }


    let has_been_deleted =
        entities::SloveneWordMutation::delete(&mut transaction, target_word_id).await?;

    if !has_been_deleted {
        return Err(APIError::not_found());
    }


    /* TODO pending cache layer rewrite
    // Signals to the the search indexer that the word has been removed.
    state
        .search
        .signal_slovene_word_removed(target_word_id)
        .await
        .map_err(APIError::InternalGenericError)?; */


    Ok(HttpResponse::Ok().finish())
}


#[rustfmt::skip]
pub fn slovene_word_router() -> Scope {
    web::scope("")
        .service(get_all_slovene_words)
        .service(create_slovene_word)
        .service(get_specific_slovene_word)
        .service(get_specific_slovene_word_by_lemma)
        .service(update_specific_slovene_word)
        .service(delete_specific_slovene_word)
}
