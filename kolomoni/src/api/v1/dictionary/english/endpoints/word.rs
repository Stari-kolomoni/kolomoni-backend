use actix_web::{delete, get, patch, post, web, Scope};
use futures_util::StreamExt;
use kolomoni_core::api_models::WordErrorReason;
use kolomoni_core::permissions::Permission;
use kolomoni_core::{
    api_models::{
        EnglishWordCreationRequest,
        EnglishWordCreationResponse,
        EnglishWordInfoResponse,
        EnglishWordUpdateRequest,
        EnglishWordsListRequest,
        EnglishWordsResponse,
    },
    ids::EnglishWordId,
};
use kolomoni_database::entities::{
    self,
    EnglishWordFieldsToUpdate,
    EnglishWordsQueryOptions,
    NewEnglishWord,
};
use sqlx::Acquire;
use tracing::info;

use crate::{
    api::{
        errors::{EndpointError, EndpointResponseBuilder, EndpointResult},
        openapi::{
            self,
            response::{requires, AsErrorReason},
        },
        traits::IntoApiModel,
        v1::dictionary::parse_uuid,
    },
    authentication::UserAuthenticationExtractor,
    declare_openapi_error_reason_response,
    require_permission_with_optional_authentication,
    require_user_authentication_and_permissions,
    state::ApplicationState,
};




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
    params(
        EnglishWordsListRequest
    ),
    responses(
        (
            status = 200,
            description = "A list of all english words.",
            body = EnglishWordsResponse,
        ),
        openapi::response::MissingPermissions<requires::WordRead, 1>,
        openapi::response::InternalServerError,
    )
)]
#[get("")]
pub async fn get_all_english_words(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    request_query_params: web::Query<EnglishWordsListRequest>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;

    require_permission_with_optional_authentication!(
        &mut database_connection,
        authentication,
        Permission::WordRead
    );



    let word_query_options = EnglishWordsQueryOptions {
        only_words_modified_after: request_query_params.into_inner().last_modified_after,
    };

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


    EndpointResponseBuilder::ok()
        .with_json_body(EnglishWordsResponse { english_words })
        .build()
}




declare_openapi_error_reason_response!(
    pub struct EnglishWordWithGivenLemmaAlreadyExists {
        description => "An english word with the given lemma already exists.",
        reason => WordErrorReason::word_with_given_lemma_already_exists()
    }
);


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
            // FIXME these error reasons aren't rendering properly in the openapi schema, fix that
            response = inline(AsErrorReason<EnglishWordWithGivenLemmaAlreadyExists>)
        ),
        openapi::response::RequiredJsonBodyErrors,
        openapi::response::MissingAuthentication,
        openapi::response::MissingPermissions<requires::WordCreate, 1>,
        openapi::response::InternalServerError,
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
    let mut database_connection = state.acquire_database_connection().await?;

    let authenticated_user = require_user_authentication_and_permissions!(
        &mut database_connection,
        authentication,
        Permission::WordCreate
    );


    let creation_request = creation_request.into_inner();


    let word_lemma_already_exists = entities::EnglishWordQuery::exists_by_exact_lemma(
        &mut database_connection,
        &creation_request.lemma,
    )
    .await?;

    if word_lemma_already_exists {
        return EndpointResponseBuilder::conflict()
            .with_error_reason(WordErrorReason::word_with_given_lemma_already_exists())
            .build();
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


    EndpointResponseBuilder::ok()
        .with_json_body(EnglishWordCreationResponse {
            // A newly-created word can not have any meanings yet.
            word: newly_created_word.into_api_model(),
        })
        .build()
}


declare_openapi_error_reason_response!(
    pub struct EnglishWordNotFound {
        description => "The requested english word does not exist.",
        reason => WordErrorReason::word_not_found()
    }
);

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
            format = Uuid,
            description = "UUID of the english word."
        )
    ),
    responses(
        (
            status = 200,
            description = "The requested english word.",
            body = EnglishWordInfoResponse,
        ),
        (
            status = 404,
            response = inline(AsErrorReason<EnglishWordNotFound>)
        ),
        openapi::response::UuidUrlParameterError,
        openapi::response::MissingPermissions<requires::WordRead, 1>,
        openapi::response::InternalServerError,
    )
)]
#[get("/{word_uuid}")]
pub async fn get_english_word_by_id(
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


    let target_english_word_id = parse_uuid::<EnglishWordId>(parameters.into_inner().0)?;


    let potential_english_word = entities::EnglishWordQuery::get_by_id_with_meanings(
        &mut database_connection,
        target_english_word_id,
    )
    .await?;

    let Some(english_word) = potential_english_word else {
        return EndpointResponseBuilder::not_found()
            .with_error_reason(WordErrorReason::word_not_found())
            .build();
    };


    EndpointResponseBuilder::ok()
        .with_json_body(EnglishWordInfoResponse {
            word: english_word.into_api_model(),
        })
        .build()
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
            description = "The requested english word.",
            body = EnglishWordInfoResponse,
        ),
        (
            status = 404,
            response = inline(AsErrorReason<EnglishWordNotFound>)
        ),
        openapi::response::UuidUrlParameterError,
        openapi::response::MissingPermissions<requires::WordRead, 1>,
        openapi::response::InternalServerError,
    )
)]
#[get("/by-lemma/{word_lemma}")]
pub async fn get_english_word_by_lemma(
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


    let target_word_lemma = parameters.into_inner().0;


    let potential_english_word = entities::EnglishWordQuery::get_by_exact_lemma_with_meanings(
        &mut database_connection,
        &target_word_lemma,
    )
    .await?;

    let Some(english_word) = potential_english_word else {
        return EndpointResponseBuilder::not_found()
            .with_error_reason(WordErrorReason::word_not_found())
            .build();
    };


    EndpointResponseBuilder::ok()
        .with_json_body(EnglishWordInfoResponse {
            word: english_word.into_api_model(),
        })
        .build()
}




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
            format = Uuid,
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
            status = 404,
            response = inline(AsErrorReason<EnglishWordNotFound>)
        ),
        openapi::response::UuidUrlParameterError,
        openapi::response::RequiredJsonBodyErrors,
        openapi::response::MissingAuthentication,
        openapi::response::MissingPermissions<requires::WordUpdate, 1>,
        openapi::response::InternalServerError,
    ),
    security(
        ("access_token" = [])
    )
)]
#[patch("/{word_uuid}")]
pub async fn update_english_word(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(String,)>,
    request_data: web::Json<EnglishWordUpdateRequest>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;
    let mut transaction = database_connection.begin().await?;

    require_user_authentication_and_permissions!(
        &mut transaction,
        authentication,
        Permission::WordUpdate
    );


    let target_word_uuid = parse_uuid::<EnglishWordId>(parameters.into_inner().0)?;

    let request_data = request_data.into_inner();



    let target_word_exists =
        entities::EnglishWordQuery::exists_by_id(&mut transaction, target_word_uuid).await?;

    if !target_word_exists {
        return EndpointResponseBuilder::not_found()
            .with_error_reason(WordErrorReason::word_not_found())
            .build();
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

        return Err(EndpointError::internal_error_with_reason(
            "Failed to update english word.",
        ));
    }



    let updated_word =
        entities::EnglishWordQuery::get_by_id_with_meanings(&mut transaction, target_word_uuid)
            .await?
            .ok_or_else(|| {
                EndpointError::internal_error_with_reason(
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

    EndpointResponseBuilder::ok()
        .with_json_body(EnglishWordInfoResponse {
            word: updated_word.into_api_model(),
        })
        .build()
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
            format = Uuid,
            description = "UUID of the english word to delete."
        )
    ),
    responses(
        (
            status = 200,
            description = "English word deleted.",
        ),
        (
            status = 404,
            response = inline(AsErrorReason<EnglishWordNotFound>)
        ),
        openapi::response::UuidUrlParameterError,
        openapi::response::MissingAuthentication,
        openapi::response::MissingPermissions<requires::WordDelete, 1>,
        openapi::response::InternalServerError,
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
    let mut database_connection = state.acquire_database_connection().await?;
    let mut transaction = database_connection.begin().await?;

    require_user_authentication_and_permissions!(
        &mut transaction,
        authentication,
        Permission::WordDelete
    );


    let target_word_uuid = parse_uuid::<EnglishWordId>(parameters.into_inner().0)?;


    let target_word_exists =
        entities::EnglishWordQuery::exists_by_id(&mut transaction, target_word_uuid).await?;

    if !target_word_exists {
        return EndpointResponseBuilder::not_found()
            .with_error_reason(WordErrorReason::word_not_found())
            .build();
    }


    let has_been_deleted =
        entities::EnglishWordMutation::delete(&mut transaction, target_word_uuid).await?;

    if !has_been_deleted {
        return Err(EndpointError::invalid_database_state(
            "failed to delete english word that had just existed in the same transaction",
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

    EndpointResponseBuilder::ok().build()
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
