use actix_http::StatusCode;
use actix_web::{delete, get, patch, post, web, HttpResponse, Scope};
use kolomoni_auth::Permission;
use kolomoni_database::{
    mutation::{CategoryMutation, NewCategory, UpdatedCategory, WordCategoryMutation},
    query::{CategoriesQueryOptions, CategoryQuery, WordCategoryQuery, WordQuery},
    shared::WordLanguage,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    api::{
        errors::{APIError, EndpointResult},
        macros::ContextlessResponder,
        openapi,
        v1::dictionary::{parse_string_into_uuid, Category},
    },
    authentication::UserAuthenticationExtractor,
    error_response_with_reason,
    impl_json_response_builder,
    require_authentication,
    require_permission,
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
        openapi::MissingOrInvalidJsonRequestBodyResponse,
        openapi::FailedAuthenticationResponses<openapi::RequiresCategoryCreate>,
        openapi::InternalServerErrorResponse,
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
    let authenticated_user = require_authentication!(authentication);
    require_permission!(
        state,
        authenticated_user,
        Permission::CategoryCreate
    );


    let request_body = request_body.into_inner();


    let exact_category_already_exists = CategoryQuery::exists_by_both_names(
        &state.database,
        request_body.slovene_name.clone(),
        request_body.english_name.clone(),
    )
    .await
    .map_err(APIError::InternalError)?;

    if exact_category_already_exists {
        return Ok(error_response_with_reason!(
            StatusCode::CONFLICT,
            "Category already exists."
        ));
    }


    let new_category = CategoryMutation::create(
        &state.database,
        NewCategory {
            english_name: request_body.english_name,
            slovene_name: request_body.slovene_name,
        },
    )
    .await
    .map_err(APIError::InternalError)?;


    state
        .search
        .signal_category_created_or_updated(new_category.id)
        .await
        .map_err(APIError::InternalError)?;


    Ok(CategoryCreationResponse {
        category: Category::from_database_model(new_category),
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
pub async fn get_all_categories(state: ApplicationState) -> EndpointResult {
    let category_models = CategoryQuery::all(&state.database, CategoriesQueryOptions::default())
        .await
        .map_err(APIError::InternalError)?;

    let categories_as_api_models = category_models
        .into_iter()
        .map(Category::from_database_model)
        .collect();


    Ok(CategoriesResponse {
        categories: categories_as_api_models,
    }
    .into_response())
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
    parameters: web::Path<(i32,)>,
) -> EndpointResult {
    let target_category_id = parameters.into_inner().0;

    let category_model = CategoryQuery::get_by_id(&state.database, target_category_id)
        .await
        .map_err(APIError::InternalError)?;


    let Some(category_model) = category_model else {
        return Err(APIError::not_found());
    };


    Ok(CategoryResponse {
        category: Category::from_database_model(category_model),
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
    pub slovene_name: Option<String>,
    pub english_name: Option<String>,
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
    parameters: web::Path<(i32,)>,
    authentication: UserAuthenticationExtractor,
    request_body: web::Json<CategoryUpdateRequest>,
) -> EndpointResult {
    let authenticated_user = require_authentication!(authentication);
    require_permission!(
        state,
        authenticated_user,
        Permission::CategoryUpdate
    );


    let request_body = request_body.into_inner();
    let target_category_id = parameters.into_inner().0;


    let target_category_before_update =
        CategoryQuery::get_by_id(&state.database, target_category_id)
            .await
            .map_err(APIError::InternalError)?;

    let Some(target_category_before_update) = target_category_before_update else {
        return Err(APIError::not_found());
    };


    let updated_category_would_conflict = CategoryQuery::exists_by_both_names(
        &state.database,
        if let Some(updated_slovene_name) = &request_body.slovene_name {
            updated_slovene_name.to_owned()
        } else {
            target_category_before_update.slovene_name
        },
        if let Some(updated_english_name) = &request_body.english_name {
            updated_english_name.to_owned()
        } else {
            target_category_before_update.english_name
        },
    )
    .await
    .map_err(APIError::InternalError)?;

    if updated_category_would_conflict {
        return Ok(error_response_with_reason!(
            StatusCode::CONFLICT,
            "Updated category would conflict with an existing category."
        ));
    }


    let updated_category = CategoryMutation::update(
        &state.database,
        target_category_id,
        UpdatedCategory {
            english_name: request_body.english_name,
            slovene_name: request_body.slovene_name,
        },
    )
    .await
    .map_err(APIError::InternalError)?;


    state
        .search
        .signal_category_created_or_updated(updated_category.id)
        .await
        .map_err(APIError::InternalError)?;


    Ok(CategoryResponse {
        category: Category::from_database_model(updated_category),
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
    parameters: web::Path<(i32,)>,
    authentication: UserAuthenticationExtractor,
) -> EndpointResult {
    let authenticated_user = require_authentication!(authentication);
    require_permission!(
        state,
        authenticated_user,
        Permission::CategoryDelete
    );


    let target_category_id = parameters.into_inner().0;

    let target_category_exists = CategoryQuery::exists_by_id(&state.database, target_category_id)
        .await
        .map_err(APIError::InternalError)?;

    if !target_category_exists {
        return Err(APIError::not_found());
    }


    CategoryMutation::delete(&state.database, target_category_id)
        .await
        .map_err(APIError::InternalError)?;


    state
        .search
        .signal_category_removed(target_category_id)
        .await
        .map_err(APIError::InternalError)?;


    Ok(HttpResponse::Ok().finish())
}




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
        .map_err(APIError::InternalError)?;
    if !target_category_exists {
        return Err(APIError::not_found_with_reason(
            "category does not exist.",
        ));
    }


    let potential_base_target_word = WordQuery::get_by_uuid(&state.database, target_word_uuid)
        .await
        .map_err(APIError::InternalError)?;

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
    .map_err(APIError::InternalError)?;
    if already_has_category {
        return Ok(error_response_with_reason!(
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
    .map_err(APIError::InternalError)?;


    // Signals to the background search indexer that the word has changed.
    match base_target_word
        .language()
        .map_err(APIError::InternalError)?
    {
        WordLanguage::Slovene => state
            .search
            .signal_slovene_word_created_or_updated(base_target_word.id)
            .await
            .map_err(APIError::InternalError)?,
        WordLanguage::English => state
            .search
            .signal_english_word_created_or_updated(base_target_word.id)
            .await
            .map_err(APIError::InternalError)?,
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
        .map_err(APIError::InternalError)?;
    if !target_category_exists {
        return Err(APIError::not_found_with_reason(
            "category does not exist.",
        ));
    }


    let target_word_exists = WordQuery::exists_by_uuid(&state.database, target_word_uuid)
        .await
        .map_err(APIError::InternalError)?;
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
    .map_err(APIError::InternalError)?;
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
    .map_err(APIError::InternalError)?;



    // Signals to the background search indexer that the word has changed.

    let base_target_word = WordQuery::get_by_uuid(&state.database, target_word_uuid)
        .await
        .map_err(APIError::InternalError)?
        .ok_or_else(|| {
            APIError::internal_reason(
                "BUG: Word dissapeared between category removal and index update.",
            )
        })?;

    match base_target_word
        .language()
        .map_err(APIError::InternalError)?
    {
        WordLanguage::Slovene => state
            .search
            .signal_slovene_word_created_or_updated(base_target_word.id)
            .await
            .map_err(APIError::InternalError)?,
        WordLanguage::English => state
            .search
            .signal_english_word_created_or_updated(base_target_word.id)
            .await
            .map_err(APIError::InternalError)?,
    };


    Ok(HttpResponse::Ok().finish())
}




#[rustfmt::skip]
pub fn categories_router() -> Scope {
    web::scope("/category")
        .service(create_category)
        .service(get_all_categories)
        .service(get_specific_category)
        .service(update_specific_category)
        .service(delete_specific_category)
        .service(link_word_to_category)
        .service(unlink_word_from_category)
}
