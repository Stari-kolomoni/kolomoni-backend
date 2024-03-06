use actix_http::StatusCode;
use actix_web::{delete, get, patch, post, web, HttpResponse, Scope};
use chrono::{DateTime, Utc};
use kolomoni_auth::Permission;
use kolomoni_database::{
    entities,
    mutation::{EnglishWordMutation, NewEnglishWord, UpdatedEnglishWord, WordMutation},
    query::{
        self,
        EnglishWordQuery,
        EnglishWordsQueryOptions,
        ExpandedEnglishWordInfo,
        RelatedEnglishWordInfo,
    },
};
use miette::Result;
use serde::{Deserialize, Serialize};
use tracing::info;
use utoipa::ToSchema;

use super::{slovene_word::SloveneWord, Category};
use crate::{
    api::{
        errors::{APIError, EndpointResult},
        macros::ContextlessResponder,
        openapi,
        v1::dictionary::parse_string_into_uuid,
    },
    authentication::UserAuthenticationExtractor,
    error_response_with_reason,
    impl_json_response_builder,
    require_authentication,
    require_permission,
    require_permission_with_optional_authentication,
    state::ApplicationState,
};



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
pub struct EnglishWord {
    /// Internal UUID of the word.
    pub id: String,

    /// An abstract or base form of the word.
    pub lemma: String,

    /// If there are multiple similar words, the disambiguation
    /// helps distinguish the word from other words at a glance.
    pub disambiguation: Option<String>,

    /// A short description of the word. Supports Markdown.
    ///
    /// TODO Will need special Markdown support for linking to other dictionary words
    ///      and possibly autocomplete in the frontend editor.
    pub description: Option<String>,

    /// When the word was created.
    pub created_at: DateTime<Utc>,

    /// When the word was last modified.
    /// This includes the last creation or deletion time of the
    /// suggestion or translation linked to this word.
    pub last_modified_at: DateTime<Utc>,

    /// A list of categories this word belongs in.
    pub categories: Vec<Category>,

    /// Suggested slovene translations of this word.
    pub suggested_translations: Vec<SloveneWord>,

    /// Slovene translations of this word.
    pub translations: Vec<SloveneWord>,
}

impl EnglishWord {
    pub fn new_without_expanded_info(english_model: entities::word_english::Model) -> Self {
        Self {
            id: english_model.word_id.to_string(),
            lemma: english_model.lemma,
            disambiguation: english_model.disambiguation,
            description: english_model.description,
            created_at: english_model.created_at.to_utc(),
            last_modified_at: english_model.last_modified_at.to_utc(),
            categories: Vec::new(),
            suggested_translations: Vec::new(),
            translations: Vec::new(),
        }
    }

    pub fn from_word_and_related_info(
        word_model: entities::word_english::Model,
        related_english_word_info: RelatedEnglishWordInfo,
    ) -> Self {
        let categories = related_english_word_info
            .categories
            .into_iter()
            .map(Category::from_database_model)
            .collect();

        let suggested_translations = related_english_word_info
            .suggested_translations
            .into_iter()
            .map(SloveneWord::from_expanded_word_info)
            .collect();

        let translations = related_english_word_info
            .translations
            .into_iter()
            .map(SloveneWord::from_expanded_word_info)
            .collect();


        Self {
            id: word_model.word_id.to_string(),
            lemma: word_model.lemma,
            disambiguation: word_model.disambiguation,
            description: word_model.description,
            created_at: word_model.created_at.to_utc(),
            last_modified_at: word_model.last_modified_at.to_utc(),
            categories,
            suggested_translations,
            translations,
        }
    }

    pub fn from_expanded_word_info(expanded_english_word_info: ExpandedEnglishWordInfo) -> Self {
        let categories = expanded_english_word_info
            .categories
            .into_iter()
            .map(Category::from_database_model)
            .collect();

        let suggested_translations = expanded_english_word_info
            .suggested_translations
            .into_iter()
            .map(SloveneWord::from_expanded_word_info)
            .collect();

        let translations = expanded_english_word_info
            .translations
            .into_iter()
            .map(SloveneWord::from_expanded_word_info)
            .collect();


        Self {
            id: expanded_english_word_info.word.word_id.to_string(),
            lemma: expanded_english_word_info.word.lemma,
            disambiguation: expanded_english_word_info.word.disambiguation,
            description: expanded_english_word_info.word.description,
            created_at: expanded_english_word_info.word.created_at.to_utc(),
            last_modified_at: expanded_english_word_info.word.last_modified_at.to_utc(),
            categories,
            suggested_translations,
            translations,
        }
    }
}


#[derive(Serialize, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Deserialize))]
pub struct EnglishWordsResponse {
    pub english_words: Vec<EnglishWord>,
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
    require_permission_with_optional_authentication!(state, authentication, Permission::WordRead);



    let word_query_options = match request_body {
        Some(body) => {
            let body = body.into_inner();

            match body.filters {
                Some(filters) => EnglishWordsQueryOptions {
                    only_words_modified_after: filters.last_modified_after,
                },
                None => EnglishWordsQueryOptions::default(),
            }
        }
        None => EnglishWordsQueryOptions::default(),
    };

    let words_with_additional_info =
        query::EnglishWordQuery::all_words_expanded(&state.database, word_query_options)
            .await
            .map_err(APIError::InternalError)?;

    let words_as_api_structures = words_with_additional_info
        .into_iter()
        .map(EnglishWord::from_expanded_word_info)
        .collect();


    Ok(EnglishWordsResponse {
        english_words: words_as_api_structures,
    }
    .into_response())
}



#[derive(Deserialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Serialize))]
#[schema(
    example = json!({
        "lemma": "adventurer",
        "disambiguation": "character",
        "description": "Playable or non-playable character.",
    })
)]
pub struct EnglishWordCreationRequest {
    pub lemma: String,
    pub disambiguation: Option<String>,
    pub description: Option<String>,
}


#[derive(Serialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Deserialize))]
#[schema(
    example = json!({
        "word": {
            "word_id": "018dbe00-266e-7398-abd2-0906df0aa345",
            "lemma": "adventurer",
            "disambiguation": "character",
            "description": "Playable or non-playable character.",
            "added_at": "2023-06-27T20:34:27.217273Z",
            "last_edited_at": "2023-06-27T20:34:27.217273Z"
        }
    })
)]
pub struct EnglishWordCreationResponse {
    pub word: EnglishWord,
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
    let authenticated_user = require_authentication!(authentication);
    require_permission!(state, authenticated_user, Permission::WordCreate);


    let creation_request = creation_request.into_inner();


    let lemma_already_exists =
        EnglishWordQuery::word_exists_by_lemma(&state.database, creation_request.lemma.clone())
            .await
            .map_err(APIError::InternalError)?;

    if lemma_already_exists {
        return Ok(error_response_with_reason!(
            StatusCode::CONFLICT,
            "An english word with the given lemma already exists."
        ));
    }


    let newly_created_word = EnglishWordMutation::create(
        &state.database,
        NewEnglishWord {
            lemma: creation_request.lemma,
            disambiguation: creation_request.disambiguation,
            description: creation_request.description,
        },
    )
    .await
    .map_err(APIError::InternalError)?;


    info!(
        created_by_user = authenticated_user.user_id(),
        "Created new english word: {}", newly_created_word.lemma,
    );


    // Signals to the the search indexer that the word has been created.
    state
        .search
        .signal_english_word_created_or_updated(newly_created_word.word_id)
        .await
        .map_err(APIError::InternalError)?;


    Ok(EnglishWordCreationResponse {
        // A newly-created word can not have any suggestions or translations yet.
        word: EnglishWord::new_without_expanded_info(newly_created_word),
    }
    .into_response())
}




#[derive(Serialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Deserialize))]
pub struct EnglishWordInfoResponse {
    pub word: EnglishWord,
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
pub async fn get_specific_english_word(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(String,)>,
) -> EndpointResult {
    require_permission_with_optional_authentication!(state, authentication, Permission::WordRead);


    let target_word_uuid = parse_string_into_uuid(&parameters.into_inner().0)?;


    let target_word = EnglishWordQuery::expanded_word_by_uuid(&state.database, target_word_uuid)
        .await
        .map_err(APIError::InternalError)?;

    let Some(target_word) = target_word else {
        return Err(APIError::not_found());
    };


    Ok(EnglishWordInfoResponse {
        word: EnglishWord::from_expanded_word_info(target_word),
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
pub async fn get_specific_english_word_by_lemma(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(String,)>,
) -> EndpointResult {
    require_permission_with_optional_authentication!(state, authentication, Permission::WordRead);


    let target_word_lemma = parameters.into_inner().0;

    let target_word = EnglishWordQuery::expanded_word_by_lemma(&state.database, target_word_lemma)
        .await
        .map_err(APIError::InternalError)?;

    let Some(target_word) = target_word else {
        return Err(APIError::not_found());
    };


    Ok(EnglishWordInfoResponse {
        word: EnglishWord::from_expanded_word_info(target_word),
    }
    .into_response())
}


#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug, ToSchema, Default)]
pub struct EnglishWordUpdateRequest {
    pub lemma: Option<String>,
    pub disambiguation: Option<String>,
    pub description: Option<String>,
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
    let authenticated_user = require_authentication!(authentication);
    require_permission!(state, authenticated_user, Permission::WordUpdate);


    let target_word_uuid = parse_string_into_uuid(&parameters.into_inner().0)?;

    let request_data = request_data.into_inner();


    let target_word_exists =
        EnglishWordQuery::word_exists_by_uuid(&state.database, target_word_uuid)
            .await
            .map_err(APIError::InternalError)?;

    if !target_word_exists {
        return Err(APIError::not_found());
    }


    let updated_model = EnglishWordMutation::update(
        &state.database,
        target_word_uuid,
        UpdatedEnglishWord {
            lemma: request_data.lemma,
            disambiguation: request_data.disambiguation,
            description: request_data.description,
        },
    )
    .await
    .map_err(APIError::InternalError)?;


    let target_word_additional_info =
        EnglishWordQuery::related_word_information_only(&state.database, target_word_uuid)
            .await
            .map_err(APIError::InternalError)?;



    // Signals to the the search indexer that the word has been updated.
    state
        .search
        .signal_english_word_created_or_updated(updated_model.word_id)
        .await
        .map_err(APIError::InternalError)?;


    Ok(EnglishWordInfoResponse {
        word: EnglishWord::from_word_and_related_info(updated_model, target_word_additional_info),
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
pub async fn delete_specific_english_word(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(String,)>,
) -> EndpointResult {
    let authenticated_user = require_authentication!(authentication);
    require_permission!(state, authenticated_user, Permission::WordDelete);


    let target_word_uuid = parse_string_into_uuid(&parameters.into_inner().0)?;

    let target_word_exists =
        EnglishWordQuery::word_exists_by_uuid(&state.database, target_word_uuid)
            .await
            .map_err(APIError::InternalError)?;

    if !target_word_exists {
        return Err(APIError::not_found());
    }


    WordMutation::delete(&state.database, target_word_uuid)
        .await
        .map_err(APIError::InternalError)?;



    // Signals to the the search indexer that the word has been removed.
    state
        .search
        .signal_english_word_removed(target_word_uuid)
        .await
        .map_err(APIError::InternalError)?;


    Ok(HttpResponse::Ok().finish())
}



// TODO Links.


#[rustfmt::skip]
pub fn english_dictionary_router() -> Scope {
    web::scope("/english")
        .service(get_all_english_words)
        .service(create_english_word)
        .service(get_specific_english_word)
        .service(get_specific_english_word_by_lemma)
        .service(update_specific_english_word)
        .service(delete_specific_english_word)
}
