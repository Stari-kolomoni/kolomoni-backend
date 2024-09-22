use actix_http::StatusCode;
use actix_web::{delete, get, patch, post, web, HttpResponse, Scope};
use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use kolomoni_auth::Permission;
use kolomoni_core::id::EnglishWordId;
use kolomoni_database::entities::{
    self,
    EnglishWordFieldsToUpdate,
    EnglishWordMeaningModelWithCategoriesAndTranslations,
    EnglishWordWithMeaningsModel,
    EnglishWordsQueryOptions,
    NewEnglishWord,
    TranslatesIntoSloveneWordModel,
};
use miette::Result;
use serde::{Deserialize, Serialize};
use sqlx::Acquire;
use tracing::info;
use utoipa::ToSchema;

use super::meaning::EnglishWordMeaningWithCategoriesAndTranslations;
use crate::{
    api::{
        errors::{APIError, EndpointResult},
        macros::ContextlessResponder,
        openapi,
        traits::IntoApiModel,
        v1::dictionary::{parse_string_into_uuid, slovene::meaning::ShallowSloveneWordMeaning},
    },
    authentication::UserAuthenticationExtractor,
    impl_json_response_builder,
    json_error_response_with_reason,
    obtain_database_connection,
    require_authentication,
    require_permission,
    require_permission_with_optional_authentication,
    state::ApplicationState,
};



// TODO needs updated example
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
#[schema(
    example = json!({
        "id": "018dbe00-266e-7398-abd2-0906df0aa345",
        "lemma": "adventurer",
        "disambiguation": "character",
        "description": "Playable or non-playable character.",
        "created_at": "2023-06-27T20:34:27.217273Z",
        "last_modified_at": "2023-06-27T20:34:27.217273Z",
        "suggested_translations": [],
        "translations": [
            {
                "id": "018dbe00-266e-7398-abd2-0906df0aa346",
                "lemma": "pustolovec",
                "disambiguation": "lik",
                "description": "Igrani ali neigrani liki, ki se odpravijo na pustolovščino.",
                "created_at": "2023-06-27T20:34:27.217273Z",
                "last_modified_at": "2023-06-27T20:34:27.217273Z"
            }
        ]
    })
)]
pub struct EnglishWordWithMeanings {
    /// Word UUID.
    pub id: EnglishWordId,

    /// An abstract or base form of the word.
    pub lemma: String,

    /// When the word was created.
    pub created_at: DateTime<Utc>,

    /// When the word was last modified.
    /// This includes the last creation or deletion time of the
    /// suggestion or translation linked to this word.
    pub last_modified_at: DateTime<Utc>,

    pub meanings: Vec<EnglishWordMeaningWithCategoriesAndTranslations>,
}


#[derive(Serialize, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Deserialize))]
pub struct EnglishWordsResponse {
    pub english_words: Vec<EnglishWordWithMeanings>,
}

impl_json_response_builder!(EnglishWordsResponse);



#[derive(Deserialize, Clone, PartialEq, Eq, Debug, ToSchema, Default)]
#[cfg_attr(feature = "with_test_facilities", derive(Serialize))]
pub struct EnglishWordFilters {
    pub last_modified_after: Option<DateTime<Utc>>,
}


#[derive(Deserialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Serialize))]
pub struct EnglishWordsListRequest {
    pub filters: Option<EnglishWordFilters>,
}

impl IntoApiModel for EnglishWordMeaningModelWithCategoriesAndTranslations {
    type ApiModel = EnglishWordMeaningWithCategoriesAndTranslations;

    fn into_api_model(self) -> Self::ApiModel {
        Self::ApiModel {
            meaning_id: self.id,
            disambiguation: self.disambiguation,
            abbreviation: self.abbreviation,
            description: self.description,
            categories: self.categories,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
            translates_into: self
                .translates_into
                .into_iter()
                .map(|internal_model| internal_model.into_api_model())
                .collect(),
        }
    }
}

impl IntoApiModel for TranslatesIntoSloveneWordModel {
    type ApiModel = ShallowSloveneWordMeaning;

    fn into_api_model(self) -> Self::ApiModel {
        Self::ApiModel {
            meaning_id: self.word_meaning_id,
            disambiguation: self.disambiguation,
            abbreviation: self.abbreviation,
            description: self.description,
            categories: self.categories,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
        }
    }
}


impl IntoApiModel for EnglishWordWithMeaningsModel {
    type ApiModel = EnglishWordWithMeanings;

    fn into_api_model(self) -> Self::ApiModel {
        let meanings = self
            .meanings
            .into_iter()
            .map(|meaning| meaning.into_api_model())
            .collect();

        Self::ApiModel {
            id: self.word_id,
            lemma: self.lemma,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
            meanings,
        }
    }
}



/// List all english words
///
/// This endpoint returns a list of all english words.
///
/// # Authentication
/// Authentication is *not required* on this endpoint due to blanket grant of
/// the `word:read` permission to unauthenticated users.
#[utoipa::path(
    get,
    path = "/dictionary/english",
    tag = "dictionary:english",
    request_body(
        content = Option<EnglishWordsListRequest>
    ),
    responses(
        (
            status = 200,
            description = "A list of all english words.",
            body = EnglishWordsResponse,
        ),
        openapi::FailedAuthenticationResponses<openapi::RequiresWordRead>,
        openapi::InternalServerErrorResponse,
    )
)]
#[get("")]
pub async fn get_all_english_words(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    request_body: Option<web::Json<EnglishWordsListRequest>>,
) -> EndpointResult {
    let mut database_connection = obtain_database_connection!(state);

    require_permission_with_optional_authentication!(
        &mut database_connection,
        authentication,
        Permission::WordRead
    );



    let word_query_options = request_body
        .map(|options| {
            options
                .into_inner()
                .filters
                .map(|filter_options| EnglishWordsQueryOptions {
                    only_words_modified_after: filter_options.last_modified_after,
                })
        })
        .flatten()
        .unwrap_or_default();

    let mut words_with_meanings_stream =
        entities::EnglishWordQuery::get_all_english_words_with_meanings(
            &mut database_connection,
            word_query_options,
        )
        .await;


    let mut english_words = Vec::new();

    while let Some(word_result) = words_with_meanings_stream.next().await {
        english_words.push(word_result?.into_api_model());
    }


    Ok(EnglishWordsResponse { english_words }.into_response())
}



#[derive(Deserialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Serialize))]
#[schema(
    example = json!({
        "lemma": "adventurer"
    })
)]
pub struct EnglishWordCreationRequest {
    pub lemma: String,
}


#[derive(Serialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Deserialize))]
#[schema(
    example = json!({
        "word": {
            "id": "018dbe00-266e-7398-abd2-0906df0aa345",
            "lemma": "adventurer",
            "added_at": "2023-06-27T20:34:27.217273Z",
            "last_edited_at": "2023-06-27T20:34:27.217273Z"
        }
    })
)]
pub struct EnglishWordCreationResponse {
    pub word: EnglishWordWithMeanings,
}

impl IntoApiModel for entities::EnglishWordModel {
    type ApiModel = EnglishWordWithMeanings;

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

impl_json_response_builder!(EnglishWordCreationResponse);


/// Create an english word
///
/// This endpoint creates a new english word in the dictionary.
///
/// # Authentication
/// This endpoint requires authentication and the `word:create` permission.
#[utoipa::path(
    post,
    path = "/dictionary/english",
    tag = "dictionary:english",
    request_body(
        content = EnglishWordCreationRequest
    ),
    responses(
        (
            status = 200,
            description = "The newly-created english word.",
            body = EnglishWordCreationResponse,
        ),
        (
            status = 409,
            description = "English word with the given lemma already exists.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "An english word with the given lemma already exists." })
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
pub async fn create_english_word(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    creation_request: web::Json<EnglishWordCreationRequest>,
) -> EndpointResult {
    let mut database_connection = obtain_database_connection!(state);

    let authenticated_user = require_authentication!(authentication);
    require_permission!(
        &mut database_connection,
        authenticated_user,
        Permission::WordCreate
    );


    let creation_request = creation_request.into_inner();


    let word_lemma_already_exists = entities::EnglishWordQuery::exists_by_exact_lemma(
        &mut database_connection,
        &creation_request.lemma,
    )
    .await?;

    if word_lemma_already_exists {
        return Ok(json_error_response_with_reason!(
            StatusCode::CONFLICT,
            "An english word with the given lemma already exists."
        ));
    }


    let newly_created_word = entities::EnglishWordMutation::create(
        &mut database_connection,
        NewEnglishWord {
            lemma: creation_request.lemma,
        },
    )
    .await?;


    info!(
        created_by_user = %authenticated_user.user_id(),
        "Created new english word: {}", newly_created_word.lemma,
    );


    /* TODO needs cache layer rework
    // Signals to the the search indexer that the word has been created.
    state
        .search
        .signal_english_word_created_or_updated(newly_created_word.word_id)
        .await
        .map_err(APIError::InternalGenericError)?; */


    Ok(EnglishWordCreationResponse {
        // A newly-created word can not have any meanings yet.
        word: newly_created_word.into_api_model(),
    }
    .into_response())
}




#[derive(Serialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Deserialize))]
pub struct EnglishWordInfoResponse {
    pub word: EnglishWordWithMeanings,
}

impl_json_response_builder!(EnglishWordInfoResponse);


/// Get an english word
///
/// This endpoint returns information about a single english word from the dictionary.
///
/// # Authentication
/// Authentication is *not required* on this endpoint due to a blanket grant of
/// the `word:read` permission to unauthenticated users.
#[utoipa::path(
    get,
    path = "/dictionary/english/{word_uuid}",
    tag = "dictionary:english",
    params(
        (
            "word_uuid" = String,
            Path,
            description = "UUID of the english word."
        )
    ),
    responses(
        (
            status = 200,
            description = "Information about the requested english word.",
            body = EnglishWordInfoResponse,
        ),
        (
            status = 400,
            description = "Invalid word UUID provided.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "Client error: invalid UUID." })
        ),
        (
            status = 404,
            description = "The requested english word does not exist."
        ),
        openapi::FailedAuthenticationResponses<openapi::RequiresWordRead>,
        openapi::InternalServerErrorResponse,
    )
)]
#[get("/{word_uuid}")]
pub async fn get_english_word_by_id(
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


    let target_word_uuid = parse_string_into_uuid(&parameters.into_inner().0)?;


    let potential_english_word = entities::EnglishWordQuery::get_by_id_with_meanings(
        &mut database_connection,
        EnglishWordId::new(target_word_uuid),
    )
    .await?;

    let Some(english_word) = potential_english_word else {
        return Err(APIError::not_found());
    };


    Ok(EnglishWordInfoResponse {
        word: english_word.into_api_model(),
    }
    .into_response())
}



/// Find an english word by lemma
///
/// This endpoint returns information about a single english word from the dictionary,
/// but takes a lemma as a parameter instead of the word ID.
///
/// Note that this is *not* intended as a search endpoint!
///
/// # Authentication
/// Authentication is *not required* on this endpoint due to a blanket grant of
/// the `word:read` permission to unauthenticated users.
#[utoipa::path(
    get,
    path = "/dictionary/english/by-lemma/{word_lemma}",
    tag = "dictionary:english",
    params(
        (
            "word_lemma" = String,
            Path,
            description = "English word lemma to look up."
        )
    ),
    responses(
        (
            status = 200,
            description = "Information about the requested english word.",
            body = EnglishWordInfoResponse,
        ),
        (
            status = 404,
            description = "The requested english word does not exist."
        ),
        openapi::FailedAuthenticationResponses<openapi::RequiresWordRead>,
        openapi::InternalServerErrorResponse,
    )
)]
#[get("/by-lemma/{word_lemma}")]
pub async fn get_english_word_by_lemma(
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


    let target_word_lemma = parameters.into_inner().0;


    let potential_english_word = entities::EnglishWordQuery::get_by_exact_lemma_with_meanings(
        &mut database_connection,
        &target_word_lemma,
    )
    .await?;

    let Some(english_word) = potential_english_word else {
        return Err(APIError::not_found());
    };


    Ok(EnglishWordInfoResponse {
        word: english_word.into_api_model(),
    }
    .into_response())
}


#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug, ToSchema, Default)]
pub struct EnglishWordUpdateRequest {
    pub lemma: Option<String>,
}

impl_json_response_builder!(EnglishWordUpdateRequest);



/// Update an english word
///
/// This endpoint updates an existing english word in the dictionary.
///
/// # Authentication
/// This endpoint requires authentication and the `word:update` permission.
#[utoipa::path(
    patch,
    path = "/dictionary/english/{word_uuid}",
    tag = "dictionary:english",
    params(
        (
            "word_uuid" = String,
            Path,
            description = "UUID of the english word."
        )
    ),
    request_body(
        content = EnglishWordUpdateRequest,
    ),
    responses(
        (
            status = 200,
            description = "Updated english word.",
            body = EnglishWordInfoResponse,
        ),
        (
            status = 400,
            description = "Invalid word UUID provided.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "Client error: invalid UUID." })
        ),
        (
            status = 404,
            description = "The requested english word does not exist."
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
pub async fn update_specific_english_word(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(String,)>,
    request_data: web::Json<EnglishWordUpdateRequest>,
) -> EndpointResult {
    let mut database_connection = obtain_database_connection!(state);
    let mut transaction = database_connection.begin().await?;

    let authenticated_user = require_authentication!(authentication);
    require_permission!(
        &mut transaction,
        authenticated_user,
        Permission::WordUpdate
    );


    let target_word_uuid = EnglishWordId::new(parse_string_into_uuid(
        &parameters.into_inner().0,
    )?);
    let request_data = request_data.into_inner();



    let target_word_exists =
        entities::EnglishWordQuery::exists_by_id(&mut transaction, target_word_uuid).await?;

    if !target_word_exists {
        return Err(APIError::not_found());
    }


    let updated_successfully = entities::EnglishWordMutation::update(
        &mut transaction,
        target_word_uuid,
        EnglishWordFieldsToUpdate {
            new_lemma: request_data.lemma,
        },
    )
    .await?;

    if !updated_successfully {
        transaction.rollback().await?;

        return Err(APIError::internal_error_with_reason(
            "Failed to update english word.",
        ));
    }



    let updated_word =
        entities::EnglishWordQuery::get_by_id_with_meanings(&mut transaction, target_word_uuid)
            .await?
            .ok_or_else(|| {
                APIError::internal_error_with_reason(
                    "Database inconsistency: word did not exist after being updated.",
                )
            })?;


    transaction.commit().await?;


    /* TODO pending rewrite of cache layer
    // Signals to the the search indexer that the word has been updated.
    state
        .search
        .signal_english_word_created_or_updated(updated_model.word_id)
        .await
        .map_err(APIError::InternalGenericError)?; */


    Ok(EnglishWordInfoResponse {
        word: updated_word.into_api_model(),
    }
    .into_response())
}



/// Delete an english word
///
/// This endpoint deletes an english word from the dictionary.
///
/// # Authentication
/// This endpoint requires authentication and the `word:delete` permission.
#[utoipa::path(
    delete,
    path = "/dictionary/english/{word_uuid}",
    tag = "dictionary:english",
    params(
        (
            "word_uuid" = String,
            Path,
            description = "UUID of the english word to delete."
        )
    ),
    responses(
        (
            status = 200,
            description = "English word deleted.",
        ),
        (
            status = 400,
            description = "Invalid word UUID provided.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "Client error: invalid UUID." })
        ),
        (
            status = 404,
            description = "The given english word does not exist."
        ),
        openapi::FailedAuthenticationResponses<openapi::RequiresWordDelete>,
        openapi::InternalServerErrorResponse,
    ),
    security(
        ("access_token" = [])
    )
)]
#[delete("/{word_uuid}")]
pub async fn delete_english_word(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(String,)>,
) -> EndpointResult {
    let mut database_connection = obtain_database_connection!(state);
    let mut transaction = database_connection.begin().await?;

    let authenticated_user = require_authentication!(authentication);
    require_permission!(
        &mut transaction,
        authenticated_user,
        Permission::WordDelete
    );


    let target_word_uuid = EnglishWordId::new(parse_string_into_uuid(
        &parameters.into_inner().0,
    )?);


    let target_word_exists =
        entities::EnglishWordQuery::exists_by_id(&mut transaction, target_word_uuid).await?;

    if !target_word_exists {
        return Err(APIError::not_found());
    }


    let has_been_deleted =
        entities::EnglishWordMutation::delete(&mut transaction, target_word_uuid).await?;

    if !has_been_deleted {
        return Err(APIError::internal_error_with_reason(
            "database inconsistency: failed to delete english word that \
            just existed in the same transaction",
        ));
    }


    /* TODO needs update when cache layer is rewritten
    // Signals to the the search indexer that the word has been removed.
    state
        .search
        .signal_english_word_removed(target_word_uuid)
        .await
        .map_err(APIError::InternalGenericError)?;
    */

    Ok(HttpResponse::Ok().finish())
}


#[rustfmt::skip]
pub fn english_word_router() -> Scope {
    web::scope("")
        .service(get_all_english_words)
        .service(create_english_word)
        .service(get_english_word_by_id)
        .service(get_english_word_by_lemma)
        // .service(update_specific_english_word)
        .service(delete_english_word)
}
