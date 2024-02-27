use actix_http::StatusCode;
use actix_web::{delete, get, patch, post, web, HttpResponse, Scope};
use chrono::{DateTime, Utc};
use kolomoni_auth::Permission;
use kolomoni_database::{
    entities,
    mutation::{NewSloveneWord, SloveneWordMutation, UpdatedSloveneWord, WordMutation},
    query::{self, SloveneWordQuery, WordCategoryQuery},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::Category;
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
        "lemma": "pustolovec",
        "disambiguation": "lik",
        "description": "Igrani ali neigrani liki, ki se odpravijo na pustolovščino.",
        "created_at": "2023-06-27T20:34:27.217273Z",
        "last_modified_at": "2023-06-27T20:34:27.217273Z"
    })
)]
pub struct SloveneWord {
    /// Internal UUID of the word.
    pub id: String,

    /// An abstract or base form of the word.
    pub lemma: String,

    /// If there are multiple similar words, the disambiguation
    /// helps distinguish the word from other words at a glance.
    pub disambiguation: Option<String>,

    /// A short description of the word. Supports Markdown.
    pub description: Option<String>,

    /// When the word was created.
    pub created_at: DateTime<Utc>,

    /// When the word was last modified.
    ///
    /// TODO In the future, this might include last modification time
    ///      of the linked suggestion and translation relationships.
    pub last_modified_at: DateTime<Utc>,

    pub categories: Vec<Category>,
}

impl SloveneWord {
    pub fn new(
        model: entities::word_slovene::Model,
        categories: Vec<entities::category::Model>,
    ) -> Self {
        let categories = categories
            .into_iter()
            .map(Category::from_database_model)
            .collect();


        Self {
            id: model.word_id.to_string(),
            lemma: model.lemma,
            disambiguation: model.disambiguation,
            description: model.description,
            created_at: model.created_at.to_utc(),
            last_modified_at: model.last_modified_at.to_utc(),
            categories,
        }
    }
}



#[derive(Serialize, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Deserialize))]
pub struct SloveneWordsResponse {
    pub slovene_words: Vec<SloveneWord>,
}

impl_json_response_builder!(SloveneWordsResponse);



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
) -> EndpointResult {
    require_permission_with_optional_authentication!(state, authentication, Permission::WordRead);

    // Load words from the database.
    let words = query::SloveneWordQuery::all_words(&state.database)
        .await
        .map_err(APIError::InternalError)?;


    let mut words_as_api_structures = Vec::with_capacity(words.len());
    for word_model in words {
        let categories =
            WordCategoryQuery::word_categories_by_word_uuid(&state.database, word_model.word_id)
                .await
                .map_err(APIError::InternalError)?;

        words_as_api_structures.push(SloveneWord::new(word_model, categories));
    }


    Ok(SloveneWordsResponse {
        slovene_words: words_as_api_structures,
    }
    .into_response())
}


#[derive(Deserialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Serialize))]
#[schema(
    example = json!({
        "lemma": "pustolovec",
        "disambiguation": "lik",
        "description": "Igrani ali neigrani liki, ki se odpravijo na pustolovščino."
    })
)]
pub struct SloveneWordCreationRequest {
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
            "lemma": "pustolovec",
            "disambiguation": "lik",
            "description": "Igrani ali neigrani liki, ki se odpravijo na pustolovščino.",
            "added_at": "2023-06-27T20:34:27.217273Z",
            "last_edited_at": "2023-06-27T20:34:27.217273Z"
        }
    })
)]
pub struct SloveneWordCreationResponse {
    pub word: SloveneWord,
}

impl_json_response_builder!(SloveneWordCreationResponse);



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
    let authenticated_user = require_authentication!(authentication);
    require_permission!(state, authenticated_user, Permission::WordCreate);


    let creation_request = creation_request.into_inner();

    let lemma_already_exists =
        SloveneWordQuery::word_exists_by_lemma(&state.database, creation_request.lemma.clone())
            .await
            .map_err(APIError::InternalError)?;

    if lemma_already_exists {
        return Ok(error_response_with_reason!(
            StatusCode::CONFLICT,
            "A slovene word with the given lemma already exists."
        ));
    }


    let newly_created_word = SloveneWordMutation::create(
        &state.database,
        NewSloveneWord {
            lemma: creation_request.lemma,
            disambiguation: creation_request.disambiguation,
            description: creation_request.description,
        },
    )
    .await
    .map_err(APIError::InternalError)?;


    Ok(SloveneWordCreationResponse {
        // Newly created words do not belong to any categories.
        word: SloveneWord::new(newly_created_word, Vec::new()),
    }
    .into_response())
}




#[derive(Serialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Deserialize))]
pub struct SloveneWordInfoResponse {
    pub word: SloveneWord,
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
    parameters: web::Path<(String,)>,
) -> EndpointResult {
    require_permission_with_optional_authentication!(state, authentication, Permission::WordRead);


    let target_word_uuid = parse_string_into_uuid(&parameters.into_inner().0)?;


    let target_word = SloveneWordQuery::word_by_uuid(&state.database, target_word_uuid)
        .await
        .map_err(APIError::InternalError)?;

    let Some(target_word) = target_word else {
        return Err(APIError::not_found());
    };


    let word_categories =
        WordCategoryQuery::word_categories_by_word_uuid(&state.database, target_word_uuid)
            .await
            .map_err(APIError::InternalError)?;


    Ok(SloveneWordInfoResponse {
        word: SloveneWord::new(target_word, word_categories),
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
    require_permission_with_optional_authentication!(state, authentication, Permission::WordRead);


    let target_word_lemma = parameters.into_inner().0;


    let target_word = SloveneWordQuery::word_by_lemma(&state.database, target_word_lemma)
        .await
        .map_err(APIError::InternalError)?;

    let Some(target_word) = target_word else {
        return Err(APIError::not_found());
    };


    let word_categories =
        WordCategoryQuery::word_categories_by_word_uuid(&state.database, target_word.word_id)
            .await
            .map_err(APIError::InternalError)?;


    Ok(SloveneWordInfoResponse {
        word: SloveneWord::new(target_word, word_categories),
    }
    .into_response())
}




#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug, ToSchema)]
pub struct SloveneWordUpdateRequest {
    pub lemma: Option<String>,
    pub disambiguation: Option<String>,
    pub description: Option<String>,
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
    parameters: web::Path<(String,)>,
    request_data: web::Json<SloveneWordUpdateRequest>,
) -> EndpointResult {
    let authenticated_user = require_authentication!(authentication);
    require_permission!(state, authenticated_user, Permission::WordUpdate);


    let target_word_uuid = parse_string_into_uuid(&parameters.into_inner().0)?;

    let request_data = request_data.into_inner();


    let target_word_exists =
        SloveneWordQuery::word_exists_by_uuid(&state.database, target_word_uuid)
            .await
            .map_err(APIError::InternalError)?;

    if !target_word_exists {
        return Err(APIError::not_found());
    }


    let updated_word = SloveneWordMutation::update(
        &state.database,
        target_word_uuid,
        UpdatedSloveneWord {
            lemma: request_data.lemma,
            disambiguation: request_data.disambiguation,
            description: request_data.description,
        },
    )
    .await
    .map_err(APIError::InternalError)?;


    let word_categories =
        WordCategoryQuery::word_categories_by_word_uuid(&state.database, target_word_uuid)
            .await
            .map_err(APIError::InternalError)?;


    Ok(SloveneWordInfoResponse {
        word: SloveneWord::new(updated_word, word_categories),
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
    parameters: web::Path<(String,)>,
) -> EndpointResult {
    let authenticated_user = require_authentication!(authentication);
    require_permission!(state, authenticated_user, Permission::WordDelete);


    let target_word_uuid = parse_string_into_uuid(&parameters.into_inner().0)?;


    let target_word_exists =
        SloveneWordQuery::word_exists_by_uuid(&state.database, target_word_uuid)
            .await
            .map_err(APIError::InternalError)?;

    if !target_word_exists {
        return Err(APIError::not_found());
    }


    WordMutation::delete(&state.database, target_word_uuid)
        .await
        .map_err(APIError::InternalError)?;


    Ok(HttpResponse::Ok().finish())
}


// TODO Links.


#[rustfmt::skip]
pub fn slovene_dictionary_router() -> Scope {
    web::scope("/slovene")
        .service(get_all_slovene_words)
        .service(create_slovene_word)
        .service(get_specific_slovene_word)
        .service(get_specific_slovene_word_by_lemma)
        .service(update_specific_slovene_word)
        .service(delete_specific_slovene_word)
}
