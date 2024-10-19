use actix_web::{delete, get, patch, post, web};
use futures_util::StreamExt;
use kolomoni_core::api_models::CategoryErrorReason;
use kolomoni_core::permissions::Permission;
use kolomoni_core::{
    api_models::{
        CategoriesResponse,
        CategoryCreationRequest,
        CategoryCreationResponse,
        CategoryResponse,
        CategoryUpdateRequest,
    },
    ids::CategoryId,
};
use kolomoni_database::entities::{self, CategoryValuesToUpdate, NewCategory};

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



declare_openapi_error_reason_response!(
    pub struct CategoryWithGivenSloveneNameAlreadyExists {
        description => "The provided slovene name for the new category \
                        is already present on an existing category.",
        reason => CategoryErrorReason::slovene_name_already_exists()
    }
);

declare_openapi_error_reason_response!(
    pub struct CategoryWithGivenEnglishNameAlreadyExists {
        description => "The provided english name for the new category \
                        is already present on an existing category.",
        reason => CategoryErrorReason::english_name_already_exists()
    }
);


/// Create a new category
///
/// This endpoint will create a new word category.
///
/// # Authentication
/// This endpoint requires authentication and the `category:create` permission.
#[utoipa::path(
    post,
    path = "/dictionary/category",
    tag = "dictionary:category",
    request_body(
        content = CategoryCreationRequest
    ),
    responses(
        (
            status = 200,
            description = "The category has been created.",
            body = CategoryCreationResponse,
        ),
        (
            status = 409,
            response = inline(AsErrorReason<CategoryWithGivenSloveneNameAlreadyExists>),
        ),
        (
            status = 409,
            response = inline(AsErrorReason<CategoryWithGivenEnglishNameAlreadyExists>),
        ),
        openapi::response::RequiredJsonBodyErrors,
        openapi::response::MissingAuthentication,
        openapi::response::MissingPermissions<requires::CategoryCreate, 1>,
        openapi::response::InternalServerError,
    ),
    security(
        ("access_token" = [])
    )
)]
#[post("")]
pub async fn create_category(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    request_body: web::Json<CategoryCreationRequest>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;
    let mut transaction = database_connection.transaction().begin().await?;

    require_user_authentication_and_permissions!(
        &mut transaction,
        authentication,
        Permission::CategoryCreate
    );


    let request_body = request_body.into_inner();


    let category_exists_by_slovene_name = entities::CategoryQuery::exists_by_slovene_name(
        &mut transaction,
        &request_body.slovene_name,
    )
    .await?;

    if category_exists_by_slovene_name {
        return EndpointResponseBuilder::conflict()
            .with_error_reason(CategoryErrorReason::slovene_name_already_exists())
            .build();
    }

    let category_exists_by_english_name = entities::CategoryQuery::exists_by_english_name(
        &mut transaction,
        &request_body.english_name,
    )
    .await?;

    if category_exists_by_english_name {
        return EndpointResponseBuilder::conflict()
            .with_error_reason(CategoryErrorReason::english_name_already_exists())
            .build();
    }



    let newly_created_category = entities::CategoryMutation::create(
        &mut transaction,
        NewCategory {
            parent_category_id: request_body.parent_category_id.map(CategoryId::new),
            slovene_name: request_body.slovene_name,
            english_name: request_body.english_name,
        },
    )
    .await?;

    /* TODO pending rewrite of cache layer
    state
        .search
        .signal_category_created_or_updated(new_category.id)
        .await
        .map_err(APIError::InternalGenericError)?; */

    EndpointResponseBuilder::ok()
        .with_json_body(CategoryCreationResponse {
            category: newly_created_category.into_api_model(),
        })
        .build()
}




/// List all word categories
///
/// This endpoint will list all word categories.
///
/// # Authentication
/// This endpoint does not require authentication.
/// It technically does require the "category:read" permission,
/// but that permission is granted to all unauthenticated API callers.
#[utoipa::path(
    get,
    path = "/dictionary/category",
    tag = "dictionary:category",
    responses(
        (
            status = 200,
            description = "The category list.",
            body = CategoriesResponse,
        ),
        openapi::response::MissingPermissions<requires::CategoryRead, 1>,
        openapi::response::InternalServerError,
    )
)]
#[get("")]
pub async fn get_all_categories(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;

    require_permission_with_optional_authentication!(
        &mut database_connection,
        authentication,
        Permission::CategoryRead
    );



    let mut category_stream =
        entities::CategoryQuery::get_all_categories(&mut database_connection).await;


    let mut categories = Vec::new();
    while let Some(internal_category) = category_stream.next().await {
        categories.push(internal_category?.into_api_model());
    }


    EndpointResponseBuilder::ok()
        .with_json_body(CategoriesResponse { categories })
        .build()
}




declare_openapi_error_reason_response!(
    pub struct CategoryIdDoesNotExist {
        description => "Category does not exist.",
        reason => CategoryErrorReason::category_not_found()
    }
);


/// Get category
///
/// This endpoint will return information about a single category.
///
/// # Authentication
/// This endpoint does not require authentication.
#[utoipa::path(
    get,
    path = "/dictionary/category/{category_uuid}",
    tag = "dictionary:category",
    params(
        (
            "category_uuid" = String,
            Path,
            format = Uuid,
            description = "UUID of the category."
        )
    ),
    responses(
        (
            status = 200,
            description = "Category information.",
            body = CategoryResponse,
        ),
        (
            status = 404,
            response = inline(AsErrorReason<CategoryIdDoesNotExist>)
        ),
        openapi::response::UuidUrlParameterError,
        openapi::response::MissingPermissions<requires::CategoryRead, 1>,
        openapi::response::InternalServerError,
    )
)]
#[get("/{category_uuid}")]
pub async fn get_specific_category(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(String,)>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;

    require_permission_with_optional_authentication!(
        &mut database_connection,
        authentication,
        Permission::CategoryRead
    );


    let target_category_id = parse_uuid::<CategoryId>(parameters.into_inner().0)?;


    let category =
        entities::CategoryQuery::get_by_id(&mut database_connection, target_category_id).await?;

    let Some(category) = category else {
        return EndpointResponseBuilder::not_found()
            .with_error_reason(CategoryErrorReason::category_not_found())
            .build();
    };


    EndpointResponseBuilder::ok()
        .with_json_body(CategoryResponse {
            category: category.into_api_model(),
        })
        .build()
}




declare_openapi_error_reason_response!(
    pub struct CategoryUpdateWouldConflictWithExistingSloveneName {
        description => "The requested update cannot be applied, because the \
                        new slovene category name is already present on another category.",
        reason => CategoryErrorReason::slovene_name_already_exists()
    }
);

declare_openapi_error_reason_response!(
    pub struct CategoryUpdateWouldConflictWithExistingEnglishName {
        description => "The requested update cannot be applied, because the \
                        new english category name is already present on another category.",
        reason => CategoryErrorReason::english_name_already_exists()
    }
);

declare_openapi_error_reason_response!(
    pub struct CategoryNoFieldsToUpdate {
        description => "Invalid request body: you should provide at least one field to update.",
        reason => CategoryErrorReason::no_fields_to_update()
    }
);



/// Update category
///
/// This endpoint allows a user with enough permissions to update a category.
///
/// # Authentication
/// This endpoint requires authentication and the `category:update` permission.
#[utoipa::path(
    patch,
    path = "/dictionary/category/{category_uuid}",
    tag = "dictionary:category",
    params(
        (
            "category_uuid" = String,
            Path,
            format = Uuid,
            description = "UUID of the category to update."
        )
    ),
    responses(
        (
            status = 200,
            description = "Updated category information.",
            body = CategoryResponse,
        ),
        (
            status = 400,
            response = inline(AsErrorReason<CategoryNoFieldsToUpdate>)
        ),
        (
            status = 404,
            response = inline(AsErrorReason<CategoryIdDoesNotExist>)
        ),
        (
            status = 409,
            response = inline(AsErrorReason<CategoryUpdateWouldConflictWithExistingEnglishName>)
        ),
        (
            status = 409,
            response = inline(AsErrorReason<CategoryUpdateWouldConflictWithExistingSloveneName>)
        ),
        openapi::response::UuidUrlParameterError,
        openapi::response::RequiredJsonBodyErrors,
        openapi::response::MissingAuthentication,
        openapi::response::MissingPermissions<requires::CategoryUpdate, 1>,
        openapi::response::InternalServerError,
    ),
    security(
        ("access_token" = [])
    )
)]
#[patch("/{category_uuid}")]
pub async fn update_specific_category(
    state: ApplicationState,
    parameters: web::Path<(String,)>,
    authentication: UserAuthenticationExtractor,
    request_body: web::Json<CategoryUpdateRequest>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;
    let mut transaction = database_connection.transaction().begin().await?;


    require_user_authentication_and_permissions!(
        &mut transaction,
        authentication,
        Permission::CategoryUpdate
    );


    let target_category_id = parse_uuid::<CategoryId>(parameters.into_inner().0)?;

    let request_body = request_body.into_inner();


    let has_no_fields_to_update = request_body.new_parent_category_id.is_none()
        && request_body.new_slovene_name.is_none()
        && request_body.new_english_name.is_none();

    if has_no_fields_to_update {
        return EndpointResponseBuilder::bad_request()
            .with_error_reason(CategoryErrorReason::no_fields_to_update())
            .build();
    }


    let target_category_exists =
        entities::CategoryQuery::exists_by_id(&mut transaction, target_category_id).await?;

    if !target_category_exists {
        return EndpointResponseBuilder::not_found()
            .with_error_reason(CategoryErrorReason::category_not_found())
            .build();
    };



    let would_conflict_by_slovene_name = if let Some(new_slovene_name) =
        request_body.new_slovene_name.as_ref()
    {
        entities::CategoryQuery::exists_by_slovene_name(&mut transaction, new_slovene_name).await?
    } else {
        false
    };

    if would_conflict_by_slovene_name {
        return EndpointResponseBuilder::conflict()
            .with_error_reason(CategoryErrorReason::slovene_name_already_exists())
            .build();
    }



    let would_conflict_by_english_name = if let Some(new_english_name) =
        request_body.new_english_name.as_ref()
    {
        entities::CategoryQuery::exists_by_english_name(&mut transaction, new_english_name).await?
    } else {
        false
    };

    if would_conflict_by_english_name {
        return EndpointResponseBuilder::conflict()
            .with_error_reason(CategoryErrorReason::english_name_already_exists())
            .build();
    }



    let successfully_updated = entities::CategoryMutation::update(
        &mut transaction,
        target_category_id,
        CategoryValuesToUpdate {
            parent_category_id: request_body
                .new_parent_category_id
                .map(|optional_id| optional_id.map(CategoryId::new)),
            slovene_name: request_body.new_slovene_name,
            english_name: request_body.new_english_name,
        },
    )
    .await?;

    if !successfully_updated {
        return Err(EndpointError::invalid_database_state(
            "failed to update a category that existed \
             in a previous call inside the same transaction",
        ));
    }


    let target_category_after_update =
        entities::CategoryQuery::get_by_id(&mut transaction, target_category_id).await?;

    let Some(target_category_after_update) = target_category_after_update else {
        return Err(EndpointError::invalid_database_state(
            "failed to fetch a category that was just updated \
             in a previous call inside the same transaction",
        ));
    };


    /* TODO pending rewrite of cache layer
    state
        .search
        .signal_category_created_or_updated(updated_category.id)
        .await
        .map_err(APIError::InternalGenericError)?; */


    EndpointResponseBuilder::ok()
        .with_json_body(CategoryResponse {
            category: target_category_after_update.into_api_model(),
        })
        .build()
}




/// Delete category
///
/// This endpoint allows a user with enough permissions to delete a category.
///
/// # Authentication
/// This endpoint requires authentication and the `category:delete` permission.
#[utoipa::path(
    delete,
    path = "/dictionary/category/{category_uuid}",
    tag = "dictionary:category",
    params(
        (
            "category_uuid" = String,
            Path,
            format = Uuid,
            description = "UUID of the category to delete."
        )
    ),
    responses(
        (
            status = 200,
            description = "Category has been deleted.",
        ),
        (
            status = 404,
            response = inline(AsErrorReason<CategoryIdDoesNotExist>)
        ),
        openapi::response::UuidUrlParameterError,
        openapi::response::MissingAuthentication,
        openapi::response::MissingPermissions<requires::CategoryDelete, 1>,
        openapi::response::InternalServerError,
    ),
    security(
        ("access_token" = [])
    )
)]
#[delete("/{category_uuid}")]
pub async fn delete_specific_category(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(String,)>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;
    let mut transaction = database_connection.transaction().begin().await?;

    require_user_authentication_and_permissions!(
        &mut transaction,
        authentication,
        Permission::CategoryDelete
    );


    let target_category_id = parse_uuid::<CategoryId>(parameters.into_inner().0)?;


    let target_category_exists =
        entities::CategoryQuery::exists_by_id(&mut transaction, target_category_id).await?;

    if !target_category_exists {
        return EndpointResponseBuilder::not_found()
            .with_error_reason(CategoryErrorReason::category_not_found())
            .build();
    }


    let successfully_deleted =
        entities::CategoryMutation::delete(&mut transaction, target_category_id).await?;

    if !successfully_deleted {
        return Err(EndpointError::invalid_database_state(
            "failed to delete a category that \
             just existed in the same transaction",
        ));
    }


    /* TODO pending rewrite of cache layer
    state
        .search
        .signal_category_removed(target_category_id)
        .await
        .map_err(APIError::InternalGenericError)?; */


    EndpointResponseBuilder::ok().build()
}



/* TODO needs to be restructured/rewritten

/// Link category to a word
///
/// This endpoint allows a user with enough permissions
/// to add a category to a word.
///
/// # Authentication
/// This endpoint requires authentication and the `word:update` permission.
#[utoipa::path(
    post,
    path = "/dictionary/category/{category_id}/word-link/{word_uuid}",
    tag = "dictionary:category",
    params(
        (
            "category_id" = i32,
            Path,
            description = "ID of the category."
        ),
        (
            "word_uuid" = String,
            Path,
            description = "ID of the word to add the category to."
        )
    ),
    responses(
        (
            status = 200,
            description = "THe category has been added to the word.",
        ),
        (
            status = 404,
            description = "Category or word does not exist."
        ),
        (
            status = 409,
            description = "This word is already linked to the provided category."
        ),
        openapi::FailedAuthenticationResponses<openapi::RequiresWordUpdate>,
        openapi::InternalServerErrorResponse,
    ),
    security(
        ("access_token" = [])
    )
)]
#[post("/{category_id}/word-link/{word_uuid}")]
pub async fn link_word_to_category(
    state: ApplicationState,
    parameters: web::Path<(i32, String)>,
    authentication: UserAuthenticationExtractor,
) -> EndpointResult {
    let authenticated_user = require_authentication!(authentication);
    require_permission!(state, authenticated_user, Permission::WordUpdate);


    let (target_category_id, target_word_uuid) = {
        let parameters = parameters.into_inner();

        let target_category_id = parameters.0;
        let target_word_uuid = parse_string_into_uuid(&parameters.1)?;

        (target_category_id, target_word_uuid)
    };


    let target_category_exists = CategoryQuery::exists_by_id(&state.database, target_category_id)
        .await
        .map_err(APIError::InternalGenericError)?;
    if !target_category_exists {
        return Err(APIError::not_found_with_reason(
            "category does not exist.",
        ));
    }


    let potential_base_target_word = WordQuery::get_by_uuid(&state.database, target_word_uuid)
        .await
        .map_err(APIError::InternalGenericError)?;

    let Some(base_target_word) = potential_base_target_word else {
        return Err(APIError::not_found_with_reason(
            "word does not exist.",
        ));
    };


    let already_has_category = WordCategoryQuery::word_has_category(
        &state.database,
        target_word_uuid,
        target_category_id,
    )
    .await
    .map_err(APIError::InternalGenericError)?;
    if already_has_category {
        return Ok(json_error_response_with_reason!(
            StatusCode::CONFLICT,
            "This category is already linked to the word."
        ));
    }


    WordCategoryMutation::add_category_to_word(
        &state.database,
        target_word_uuid,
        target_category_id,
    )
    .await
    .map_err(APIError::InternalGenericError)?;


    // Signals to the background search indexer that the word has changed.
    match base_target_word
        .language()
        .map_err(APIError::InternalGenericError)?
    {
        WordLanguage::Slovene => state
            .search
            .signal_slovene_word_created_or_updated(base_target_word.id)
            .await
            .map_err(APIError::InternalGenericError)?,
        WordLanguage::English => state
            .search
            .signal_english_word_created_or_updated(base_target_word.id)
            .await
            .map_err(APIError::InternalGenericError)?,
    };


    Ok(HttpResponse::Ok().finish())
}



/// Unlink a category from a word
///
/// This endpoint allows a user with enough permissions
/// to remove a category from a word.
///
/// # Authentication
/// This endpoint requires authentication and the `word:update` permission.
#[utoipa::path(
    delete,
    path = "/dictionary/category/{category_id}/word-link/{word_uuid}",
    tag = "dictionary:category",
    params(
        (
            "category_id" = i32,
            Path,
            description = "ID of the category."
        ),
        (
            "word_uuid" = String,
            Path,
            description = "ID of the word to add the category to."
        )
    ),
    responses(
        (
            status = 200,
            description = "THe category has been added to the word.",
        ),
        (
            status = 404,
            description = "Category or word does not exist OR the word does not have the specified category."
        ),
        openapi::MissingOrInvalidJsonRequestBodyResponse,
        openapi::FailedAuthenticationResponses<openapi::RequiresWordUpdate>,
        openapi::InternalServerErrorResponse,
    ),
    security(
        ("access_token" = [])
    )
)]
#[delete("/{category_id}/word-link/{word_uuid}")]
pub async fn unlink_word_from_category(
    state: ApplicationState,
    parameters: web::Path<(i32, String)>,
    authentication: UserAuthenticationExtractor,
) -> EndpointResult {
    let authenticated_user = require_authentication!(authentication);
    require_permission!(state, authenticated_user, Permission::WordUpdate);


    let (target_category_id, target_word_uuid) = {
        let parameters = parameters.into_inner();

        let target_category_id = parameters.0;
        let target_word_uuid = parse_string_into_uuid(&parameters.1)?;

        (target_category_id, target_word_uuid)
    };


    let target_category_exists = CategoryQuery::exists_by_id(&state.database, target_category_id)
        .await
        .map_err(APIError::InternalGenericError)?;
    if !target_category_exists {
        return Err(APIError::not_found_with_reason(
            "category does not exist.",
        ));
    }


    let target_word_exists = WordQuery::exists_by_uuid(&state.database, target_word_uuid)
        .await
        .map_err(APIError::InternalGenericError)?;
    if !target_word_exists {
        return Err(APIError::not_found_with_reason(
            "word does not exist.",
        ));
    }


    let category_link_exists = WordCategoryQuery::word_has_category(
        &state.database,
        target_word_uuid,
        target_category_id,
    )
    .await
    .map_err(APIError::InternalGenericError)?;
    if !category_link_exists {
        return Err(APIError::not_found_with_reason(
            "the word isn't linked to this category.",
        ));
    }


    WordCategoryMutation::remove_category_from_word(
        &state.database,
        target_word_uuid,
        target_category_id,
    )
    .await
    .map_err(APIError::InternalGenericError)?;



    // Signals to the background search indexer that the word has changed.

    let base_target_word = WordQuery::get_by_uuid(&state.database, target_word_uuid)
        .await
        .map_err(APIError::InternalGenericError)?
        .ok_or_else(|| {
            APIError::internal_error_with_reason(
                "BUG: Word disappeared between category removal and index update.",
            )
        })?;

    match base_target_word
        .language()
        .map_err(APIError::InternalGenericError)?
    {
        WordLanguage::Slovene => state
            .search
            .signal_slovene_word_created_or_updated(base_target_word.id)
            .await
            .map_err(APIError::InternalGenericError)?,
        WordLanguage::English => state
            .search
            .signal_english_word_created_or_updated(base_target_word.id)
            .await
            .map_err(APIError::InternalGenericError)?,
    };


    Ok(HttpResponse::Ok().finish())
}


 */
