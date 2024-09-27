use actix_http::StatusCode;
use actix_web::{delete, get, patch, post, web, HttpResponse, Scope};
use futures_util::StreamExt;
use kolomoni_auth::Permission;
use kolomoni_core::{
    api_models::{
        SloveneWordCreationRequest,
        SloveneWordCreationResponse,
        SloveneWordInfoResponse,
        SloveneWordUpdateRequest,
        SloveneWordsListRequest,
        SloveneWordsResponse,
    },
    id::SloveneWordId,
};
use kolomoni_database::entities::{
    self,
    NewSloveneWord,
    SloveneWordFieldsToUpdate,
    SloveneWordsQueryOptions,
};
use sqlx::Acquire;
use tracing::info;
use uuid::Uuid;

use crate::{
    api::{
        errors::{APIError, EndpointResult},
        macros::ContextlessResponder,
        openapi,
        traits::IntoApiModel,
        v1::dictionary::parse_string_into_uuid,
    },
    authentication::UserAuthenticationExtractor,
    json_error_response_with_reason,
    obtain_database_connection,
    require_permission_with_optional_authentication,
    require_user_authentication_and_permission,
    state::ApplicationState,
};




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
        openapi::response::FailedAuthentication<openapi::response::requires::WordRead>,
        openapi::response::InternalServerError,
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


    let word_query_options = request_body
        .and_then(|options| {
            options
                .into_inner()
                .filters
                .map(|filter_options| SloveneWordsQueryOptions {
                    only_words_modified_after: filter_options.last_modified_after,
                })
        })
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
        openapi::response::MissingOrInvalidJsonRequestBody,
        openapi::response::FailedAuthentication<openapi::response::requires::WordCreate>,
        openapi::response::InternalServerError,
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

    let authenticated_user = require_user_authentication_and_permission!(
        &mut transaction,
        authentication,
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
        openapi::response::FailedAuthentication<openapi::response::requires::WordRead>,
        openapi::response::InternalServerError,
    )
)]
#[get("/{word_uuid}")]
pub async fn get_specific_slovene_word(
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
        openapi::response::FailedAuthentication<openapi::response::requires::WordRead>,
        openapi::response::InternalServerError,
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
        target_word_lemma,
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
        openapi::response::MissingOrInvalidJsonRequestBody,
        openapi::response::FailedAuthentication<openapi::response::requires::WordUpdate>,
        openapi::response::InternalServerError,
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
    let mut database_connection = obtain_database_connection!(state);
    let mut transaction = database_connection.begin().await?;

    require_user_authentication_and_permission!(
        &mut transaction,
        authentication,
        Permission::WordUpdate
    );


    let target_word_id = SloveneWordId::new(parse_string_into_uuid(
        parameters.into_inner().0.as_str(),
    )?);

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
            format = Uuid,
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
        openapi::response::FailedAuthentication<openapi::response::requires::WordDelete>,
        openapi::response::InternalServerError,
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
    let mut database_connection = obtain_database_connection!(state);
    let mut transaction = database_connection.begin().await?;

    require_user_authentication_and_permission!(
        &mut transaction,
        authentication,
        Permission::WordDelete
    );


    let target_word_id = SloveneWordId::new(parse_string_into_uuid(
        parameters.into_inner().0.as_str(),
    )?);


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
