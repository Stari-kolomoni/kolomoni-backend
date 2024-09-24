use actix_http::StatusCode;
use actix_web::{delete, get, patch, post, web, HttpResponse, Scope};
use futures_util::StreamExt;
use kolomoni_auth::Permission;
use kolomoni_core::id::CategoryId;
use kolomoni_database::entities::{self, CategoryValuesToUpdate, NewCategory};
use serde::{Deserialize, Serialize};
use sqlx::Acquire;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    api::{
        errors::{APIError, EndpointResult},
        macros::ContextlessResponder,
        openapi,
        traits::IntoApiModel,
        v1::dictionary::Category,
    },
    authentication::UserAuthenticationExtractor,
    impl_json_response_builder,
    json_error_response_with_reason,
    obtain_database_connection,
    require_permission_OLD,
    require_permission_with_optional_authentication,
    require_user_authentication,
    state::ApplicationState,
};




#[derive(Deserialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Serialize))]
#[schema(
    example = json!({
        "slovene_name": "Dejavnosti in spopad",
        "english_name": "Activities and Combat",
    })
)]
pub struct CategoryCreationRequest {
    pub parent_category_id: Option<Uuid>,
    pub slovene_name: String,
    pub english_name: String,
}


#[derive(Serialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Deserialize))]
#[schema(
    example = json!({
        "category": {
            "id": 1,
            "slovene_name": "Dejavnosti in spopad",
            "english_name": "Activities and Combat",
            "created_at": "2023-06-27T20:34:27.217273Z",
            "last_modified_at": "2023-06-27T20:34:27.217273Z",
        }
    })
)]
pub struct CategoryCreationResponse {
    pub category: Category,
}

impl_json_response_builder!(CategoryCreationResponse);


impl IntoApiModel for entities::CategoryModel {
    type ApiModel = Category;

    fn into_api_model(self) -> Self::ApiModel {
        Category {
            id: self.id,
            english_name: self.english_name,
            slovene_name: self.slovene_name,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
        }
    }
}



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
            description = "This english-slovene word combination already exists as a category."
        ),
        openapi::response::MissingOrInvalidJsonRequestBodyResponse,
        openapi::response::FailedAuthenticationResponses<openapi::response::RequiresCategoryCreate>,
        openapi::response::InternalServerErrorResponse,
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
    let mut database_connection = obtain_database_connection!(state);
    let mut transaction = database_connection.begin().await?;

    let authenticated_user = require_user_authentication!(authentication);
    require_permission_OLD!(
        &mut transaction,
        authenticated_user,
        Permission::CategoryCreate
    );


    let request_body = request_body.into_inner();


    let category_exists_by_slovene_name = entities::CategoryQuery::exists_by_slovene_name(
        &mut transaction,
        &request_body.slovene_name,
    )
    .await?;

    if category_exists_by_slovene_name {
        return Ok(json_error_response_with_reason!(
            StatusCode::CONFLICT,
            "Category with given slovene name already exists."
        ));
    }

    let category_exists_by_english_name = entities::CategoryQuery::exists_by_english_name(
        &mut transaction,
        &request_body.english_name,
    )
    .await?;

    if category_exists_by_english_name {
        return Ok(json_error_response_with_reason!(
            StatusCode::CONFLICT,
            "Category with given english name already exists."
        ));
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


    Ok(CategoryCreationResponse {
        category: newly_created_category.into_api_model(),
    }
    .into_response())
}




#[derive(Serialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Deserialize))]
pub struct CategoriesResponse {
    pub categories: Vec<Category>,
}

impl_json_response_builder!(CategoriesResponse);



/// List all word categories
///
/// This endpoint will list all word categories.
///
/// # Authentication
/// This endpoint does not require authentication.
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
        openapi::InternalServerErrorResponse,
    )
)]
#[get("")]
pub async fn get_all_categories(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
) -> EndpointResult {
    let mut database_connection = obtain_database_connection!(state);

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


    Ok(CategoriesResponse { categories }.into_response())
}




#[derive(Serialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Deserialize))]
#[schema(
    example = json!({
        "category": {
            "id": 1,
            "slovene_name": "Dejavnosti in spopad",
            "english_name": "Activities and Combat",
        }
    })
)]
pub struct CategoryResponse {
    pub category: Category,
}

impl_json_response_builder!(CategoryResponse);


/// Get category
///
/// This endpoint will return information about a single category.
///
/// # Authentication
/// This endpoint does not require authentication.
#[utoipa::path(
    get,
    path = "/dictionary/category/{category_id}",
    tag = "dictionary:category",
    params(
        (
            "category_id" = i32,
            Path,
            description = "ID of the category."
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
            description = "Category does not exist."
        ),
        openapi::InternalServerErrorResponse,
    )
)]
#[get("/{category_id}")]
pub async fn get_specific_category(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    parameters: web::Path<(Uuid,)>,
) -> EndpointResult {
    let mut database_connection = obtain_database_connection!(state);

    require_permission_with_optional_authentication!(
        &mut database_connection,
        authentication,
        Permission::CategoryRead
    );


    let target_category_id = CategoryId::new(parameters.into_inner().0);


    let category =
        entities::CategoryQuery::get_by_id(&mut database_connection, target_category_id).await?;

    let Some(category) = category else {
        return Err(APIError::not_found());
    };


    Ok(CategoryResponse {
        category: category.into_api_model(),
    }
    .into_response())
}



#[derive(Deserialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Serialize))]
#[schema(
    example = json!({
        "slovene_name": "Dejavnosti in spopad",
        "english_name": "Activities and Combat",
    })
)]
pub struct CategoryUpdateRequest {
    /// # Interpreting the double option
    /// To distinguish from an unset and a null JSON value, this field is a
    /// double option. `None` indicates the field was not present
    /// (i.e. that the parent category should not change as part of this update),
    /// while `Some(None)` indicates it was set to `null`
    /// (i.e. that the parent category should be cleared).
    ///
    /// See also: [`serde_with::rust::double_option`].
    #[serde(default, with = "::serde_with::rust::double_option")]
    pub new_parent_category_id: Option<Option<Uuid>>,

    pub new_slovene_name: Option<String>,

    pub new_english_name: Option<String>,
}



/// Update category
///
/// This endpoint allows a user with enough permissions to update a category.
///
/// # Authentication
/// This endpoint requires authentication and the `category:update` permission.
#[utoipa::path(
    patch,
    path = "/dictionary/category/{category_id}",
    tag = "dictionary:category",
    params(
        (
            "category_id" = i32,
            Path,
            description = "ID of the category to update."
        )
    ),
    responses(
        (
            status = 200,
            description = "Updated category information.",
            body = CategoryResponse,
        ),
        (
            status = 404,
            description = "Category does not exist."
        ),
        (
            status = 409,
            description = "The update would create a conflict with another category."
        ),
        openapi::MissingOrInvalidJsonRequestBodyResponse,
        openapi::FailedAuthenticationResponses<openapi::RequiresCategoryUpdate>,
        openapi::InternalServerErrorResponse,
    ),
    security(
        ("access_token" = [])
    )
)]
#[patch("/{category_id}")]
pub async fn update_specific_category(
    state: ApplicationState,
    parameters: web::Path<(Uuid,)>,
    authentication: UserAuthenticationExtractor,
    request_body: web::Json<CategoryUpdateRequest>,
) -> EndpointResult {
    let mut database_connection = obtain_database_connection!(state);
    let mut transaction = database_connection.begin().await?;


    let authenticated_user = require_user_authentication!(authentication);
    require_permission_OLD!(
        &mut transaction,
        authenticated_user,
        Permission::CategoryUpdate
    );


    let target_category_id = CategoryId::new(parameters.into_inner().0);
    let request_body = request_body.into_inner();


    let has_no_fields_to_update = request_body.new_parent_category_id.is_none()
        && request_body.new_slovene_name.is_none()
        && request_body.new_english_name.is_none();

    if has_no_fields_to_update {
        return Ok(json_error_response_with_reason!(
            StatusCode::BAD_REQUEST,
            "Client should provide at least one category field to update; \
            providing none on this endpoint is invalid."
        ));
    }


    let target_category_exists =
        entities::CategoryQuery::exists_by_id(&mut transaction, target_category_id).await?;

    if !target_category_exists {
        return Err(APIError::not_found());
    };



    let would_conflict_by_slovene_name = if let Some(new_slovene_name) =
        request_body.new_slovene_name.as_ref()
    {
        entities::CategoryQuery::exists_by_slovene_name(&mut transaction, &new_slovene_name).await?
    } else {
        false
    };

    if would_conflict_by_slovene_name {
        return Ok(json_error_response_with_reason!(
            StatusCode::CONFLICT,
            "Updated category would conflict with an existing category by its slovene name."
        ));
    }



    let would_conflict_by_english_name = if let Some(new_english_name) =
        request_body.new_english_name.as_ref()
    {
        entities::CategoryQuery::exists_by_english_name(&mut transaction, &new_english_name).await?
    } else {
        false
    };

    if would_conflict_by_english_name {
        return Ok(json_error_response_with_reason!(
            StatusCode::CONFLICT,
            "Updated category would conflict with an existing category by its english name."
        ));
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
        return Err(APIError::internal_error_with_reason(
            "database inconsistency: failed to update a category \
            that existed in a previous call inside the same transaction",
        ));
    }


    let target_category_after_update =
        entities::CategoryQuery::get_by_id(&mut transaction, target_category_id).await?;

    let Some(target_category_after_update) = target_category_after_update else {
        return Err(APIError::internal_error_with_reason(
            "database inconsistency: failed to fetch a category \
            that was just updated in a previous call inside the same transaction",
        ));
    };


    /* TODO pending rewrite of cache layer
    state
        .search
        .signal_category_created_or_updated(updated_category.id)
        .await
        .map_err(APIError::InternalGenericError)?; */


    Ok(CategoryResponse {
        category: target_category_after_update.into_api_model(),
    }
    .into_response())
}




/// Delete category
///
/// This endpoint allows a user with enough permissions to delete a category.
///
/// # Authentication
/// This endpoint requires authentication and the `category:delete` permission.
#[utoipa::path(
    delete,
    path = "/dictionary/category/{category_id}",
    tag = "dictionary:category",
    params(
        (
            "category_id" = i32,
            Path,
            description = "ID of the category to delete."
        )
    ),
    responses(
        (
            status = 200,
            description = "Category has been deleted.",
        ),
        (
            status = 404,
            description = "Category does not exist."
        ),
        openapi::FailedAuthenticationResponses<openapi::RequiresCategoryDelete>,
        openapi::InternalServerErrorResponse,
    ),
    security(
        ("access_token" = [])
    )
)]
#[delete("/{category_id}")]
pub async fn delete_specific_category(
    state: ApplicationState,
    parameters: web::Path<(Uuid,)>,
    authentication: UserAuthenticationExtractor,
) -> EndpointResult {
    let mut database_connection = obtain_database_connection!(state);
    let mut transaction = database_connection.begin().await?;

    let authenticated_user = require_user_authentication!(authentication);
    require_permission_OLD!(
        &mut transaction,
        authenticated_user,
        Permission::CategoryDelete
    );


    let target_category_id = CategoryId::new(parameters.into_inner().0);



    let target_category_exists =
        entities::CategoryQuery::exists_by_id(&mut transaction, target_category_id).await?;

    if !target_category_exists {
        return Err(APIError::not_found());
    }


    let successfully_deleted =
        entities::CategoryMutation::delete(&mut transaction, target_category_id).await?;

    if !successfully_deleted {
        return Err(APIError::internal_error_with_reason(
            "database inconsistency: failed to delete a category that \
            previously existed in the same transaction",
        ));
    }


    /* TODO pending rewrite of cache layer
    state
        .search
        .signal_category_removed(target_category_id)
        .await
        .map_err(APIError::InternalGenericError)?; */


    Ok(HttpResponse::Ok().finish())
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
                "BUG: Word dissapeared between category removal and index update.",
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

#[rustfmt::skip]
pub fn categories_router() -> Scope {
    web::scope("/category")
        .service(create_category)
        .service(get_all_categories)
        .service(get_specific_category)
        .service(update_specific_category)
        .service(delete_specific_category)
        // .service(link_word_to_category)
        // .service(unlink_word_from_category)
}
