use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{delete, HttpResponseBuilder};
use actix_web::{get, patch, post, web, HttpRequest, HttpResponse, Responder, Scope};
use anyhow::Result;
use chrono::{DateTime, Utc};
use sea_orm::TransactionTrait;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::api::auth::{UserAuth, UserPermission, UserPermissions};
use crate::api::errors::{APIError, EndpointResult, ErrorReasonResponse};
use crate::api::macros::DumbResponder;
use crate::database::mutation::users::UserRegistrationInfo;
use crate::database::{entities, mutation, queries};
use crate::state::AppState;
use crate::{impl_json_responder, require_permission, response_with_reason};

/*
 * Shared
 */

#[derive(Serialize, Debug)]
pub struct PublicUserModel {
    pub id: i32,
    pub username: String,
    pub display_name: String,
    pub joined_at: DateTime<Utc>,
    pub last_modified_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
}

impl PublicUserModel {
    #[inline]
    pub fn from_user_model(model: entities::users::Model) -> Self {
        Self {
            id: model.id,
            username: model.username,
            display_name: model.display_name,
            joined_at: model.joined_at,
            last_modified_at: model.last_modified_at,
            last_active_at: model.last_active_at,
        }
    }
}



#[derive(Serialize, Debug)]
pub struct UserInfoResponse {
    pub user: PublicUserModel,
}

impl UserInfoResponse {
    pub fn new(model: entities::users::Model) -> Self {
        Self {
            user: PublicUserModel::from_user_model(model),
        }
    }
}

impl_json_responder!(UserInfoResponse);



#[derive(Deserialize, Clone, Debug)]
pub struct UserDisplayNameChangeRequest {
    pub new_display_name: String,
}


#[derive(Serialize, Debug)]
pub struct UserDisplayNameChangeResponse {
    pub user: PublicUserModel,
}

impl_json_responder!(UserDisplayNameChangeResponse);


#[derive(Serialize, Debug)]
pub struct UserPermissionsResponse {
    pub permissions: Vec<String>,
}

impl_json_responder!(UserPermissionsResponse);



/*
 * POST /
 */

#[derive(Deserialize, Clone, Debug)]
pub struct UserRegistrationData {
    pub username: String,
    pub display_name: String,
    pub password: String,
}

impl From<UserRegistrationData> for UserRegistrationInfo {
    fn from(value: UserRegistrationData) -> Self {
        Self {
            username: value.username,
            display_name: value.display_name,
            password: value.password,
        }
    }
}


#[derive(Serialize, Debug)]
pub struct UserRegistrationResponse {
    pub user: PublicUserModel,
}

impl_json_responder!(UserRegistrationResponse);


#[post("/")]
pub async fn register_user(
    state: web::Data<AppState>,
    json_data: web::Json<UserRegistrationData>,
) -> EndpointResult {
    let username_already_exists =
        queries::users::Query::user_exists_by_username(&state.database, &json_data.username)
            .await
            .map_err(APIError::InternalError)?;

    if username_already_exists {
        return Ok(response_with_reason!(
            StatusCode::CONFLICT,
            "User with provided username already exists."
        ));
    }


    let new_user = mutation::users::Mutation::create_user(
        &state.database,
        &state.hasher,
        json_data.clone().into(),
    )
    .await
    .map_err(APIError::InternalError)?;

    Ok(UserRegistrationResponse {
        user: PublicUserModel::from_user_model(new_user),
    }
    .into_response())
}



/*
 * GET /me
 */

#[get("/me")]
pub async fn get_current_user_info(
    user_auth: UserAuth,
    state: web::Data<AppState>,
) -> EndpointResult {
    let (token, permissions) = user_auth
        .token_and_permissions_if_authenticated(&state.database)
        .await
        .map_err(APIError::InternalError)?
        .ok_or_else(|| APIError::NotAuthenticated)?;

    // User must have the `user.self:read` permission to access this endpoint.
    require_permission!(permissions, UserPermission::UserSelfRead);


    let user = queries::users::Query::get_user_by_username(&state.database, &token.username)
        .await
        .map_err(APIError::InternalError)?
        .ok_or_else(|| APIError::not_found())?;


    Ok(UserInfoResponse::new(user).into_response())
}



/*
 * GET /me/permissions
 */

#[get("/me/permissions")]
async fn get_current_user_permissions(
    user_auth: UserAuth,
    state: web::Data<AppState>,
) -> EndpointResult {
    let permissions = user_auth
        .permissions_if_authenticated(&state.database)
        .await
        .map_err(APIError::InternalError)?
        .ok_or_else(|| APIError::NotAuthenticated)?;

    // User must have the `user.self:read` permission to access this endpoint.
    require_permission!(permissions, UserPermission::UserSelfRead);

    Ok(UserPermissionsResponse {
        permissions: permissions.to_vec_of_permission_names(),
    }
    .into_response())
}



/*
 * PATCH /me/display_name
 */

#[patch("/me/display_name")]
async fn update_current_user_display_name(
    user_auth: UserAuth,
    state: web::Data<AppState>,
    json_data: web::Json<UserDisplayNameChangeRequest>,
) -> EndpointResult {
    // TODO Rate-limiting.

    let (token, permissions) = user_auth
        .token_and_permissions_if_authenticated(&state.database)
        .await
        .map_err(APIError::InternalError)?
        .ok_or_else(|| APIError::NotAuthenticated)?;

    // Users must have the `user.self:write` permission to access this endpoint.
    require_permission!(permissions, UserPermission::UserSelfWrite);

    let json_data = json_data.into_inner();

    let database_transaction = state
        .database
        .begin()
        .await
        .map_err(APIError::InternalDatabaseError)?;

    mutation::users::Mutation::update_display_name_by_username(
        &database_transaction,
        &token.username,
        json_data.new_display_name.clone(),
    )
    .await
    .map_err(APIError::InternalError)?;

    // TODO Consider merging this update into all mutation methods where it makes sense.
    //      Otherwise we're wasting a round-trip to the database for no real reason.
    let updated_user = mutation::users::Mutation::update_last_active_at_by_username(
        &database_transaction,
        &token.username,
        None,
    )
    .await
    .map_err(APIError::InternalError)?;

    database_transaction
        .commit()
        .await
        .map_err(APIError::InternalDatabaseError)?;


    info!(
        username = token.username,
        new_display_name = json_data.new_display_name,
        "User has updated their display name."
    );

    Ok(UserDisplayNameChangeResponse {
        user: PublicUserModel::from_user_model(updated_user),
    }
    .into_response())
}



/*
 * GET /{id}
 */

#[get("/{user_id}")]
async fn get_specific_user_info(
    user_auth: UserAuth,
    path_info: web::Path<(i32,)>,
    state: web::Data<AppState>,
) -> EndpointResult {
    let requested_user_id = path_info.into_inner().0;

    // Only authenticated users with the `user.any:read` permission to access this endpoint.
    let permissions = user_auth
        .permissions_if_authenticated(&state.database)
        .await
        .map_err(APIError::InternalError)?
        .ok_or_else(|| APIError::NotAuthenticated)?;

    require_permission!(permissions, UserPermission::UserAnyRead);

    // Return information about the requested user.
    let optional_user = queries::users::Query::get_user_by_id(&state.database, requested_user_id)
        .await
        .map_err(APIError::InternalError)?;

    let Some(user) = optional_user else {
        return Ok(HttpResponse::NotFound().finish());
    };

    Ok(UserInfoResponse::new(user).into_response())
}



/*
 * GET /{id}/permissions
 */

#[get("/{user_id}/permissions")]
async fn get_specific_user_permissions(
    user_auth: UserAuth,
    path_info: web::Path<(i32,)>,
    state: web::Data<AppState>,
) -> EndpointResult {
    let requested_user_id = path_info.into_inner().0;

    // Only authenticated users with the `user.any:read` permission to access this endpoint.
    let permissions = user_auth
        .permissions_if_authenticated(&state.database)
        .await
        .map_err(APIError::InternalError)?
        .ok_or_else(|| APIError::NotAuthenticated)?;

    require_permission!(permissions, UserPermission::UserAnyRead);

    // Get user permissions.
    let optional_user_permissions =
        queries::user_permissions::Query::get_user_permission_names_by_user_id(
            &state.database,
            requested_user_id,
        )
        .await
        .map_err(APIError::InternalError)?;

    let Some(permissions) = optional_user_permissions else {
        return Ok(HttpResponse::NotFound().finish());
    };

    Ok(UserPermissionsResponse { permissions }.into_response())
}



/*
 * PATCH /{user_id}/display_name
 */

#[patch("/{user_id}/display_name")]
async fn update_specific_user_display_name(
    user_auth: UserAuth,
    path_info: web::Path<(i32,)>,
    state: web::Data<AppState>,
    json_data: web::Json<UserDisplayNameChangeRequest>,
) -> EndpointResult {
    let requested_user_id = path_info.into_inner().0;

    // Only authenticated users with the `user.any:write` permission can modify
    // others' display names. Intended for moderation tooling.
    let (token, permissions) = user_auth
        .token_and_permissions_if_authenticated(&state.database)
        .await
        .map_err(APIError::InternalError)?
        .ok_or_else(|| APIError::NotAuthenticated)?;

    require_permission!(permissions, UserPermission::UserAnyWrite);

    let json_data = json_data.into_inner();


    let database_transaction = state
        .database
        .begin()
        .await
        .map_err(APIError::InternalDatabaseError)?;

    mutation::users::Mutation::update_display_name_by_user_id(
        &database_transaction,
        requested_user_id,
        json_data.new_display_name.clone(),
    )
    .await
    .map_err(APIError::InternalError)?;

    let updated_user = mutation::users::Mutation::update_last_active_at_by_user_id(
        &database_transaction,
        requested_user_id,
        None,
    )
    .await
    .map_err(APIError::InternalError)?;

    database_transaction
        .commit()
        .await
        .map_err(APIError::InternalDatabaseError)?;


    info!(
        operator = token.username,
        target_user_id = requested_user_id,
        new_display_name = json_data.new_display_name,
        "User has updated another user's display name."
    );

    Ok(UserDisplayNameChangeResponse {
        user: PublicUserModel::from_user_model(updated_user),
    }
    .into_response())
}



/*
 * POST /{user_id}/permissions
 */

#[derive(Deserialize)]
pub struct UserPermissionAddRequest {
    pub permissions_to_add: Vec<String>,
}


#[post("/{user_id}/permissions")]
async fn add_permissions_to_specific_user(
    user_auth: UserAuth,
    path_info: web::Path<(i32,)>,
    state: web::Data<AppState>,
    json_data: web::Json<UserPermissionAddRequest>,
) -> EndpointResult {
    let requested_user_id = path_info.into_inner().0;

    // Only authenticated users with the `user.any:write` permission can add permissions
    // to other users, but only if they also have the requested permission.
    // Intended for moderation tooling.
    let current_user_permissions = user_auth
        .permissions_if_authenticated(&state.database)
        .await
        .map_err(APIError::InternalError)?
        .ok_or_else(|| APIError::NotAuthenticated)?;

    require_permission!(
        current_user_permissions,
        UserPermission::UserAnyWrite
    );


    let json_data = json_data.into_inner();

    let permissions_to_add_result: Result<Vec<UserPermission>, &str> = json_data
        .permissions_to_add
        .iter()
        .map(|permission_name| {
            UserPermission::from_name(permission_name.as_str())
                .ok_or_else(|| permission_name.as_str())
        })
        .collect::<Result<Vec<UserPermission>, &str>>();

    let permissions_to_add = match permissions_to_add_result {
        Ok(permissions_to_add) => permissions_to_add,
        Err(non_existent_permission_name) => {
            return Ok(response_with_reason!(
                StatusCode::BAD_REQUEST,
                format!("No such permission: {non_existent_permission_name}")
            ))
        }
    };

    // Validate that the current user has all of the permissions
    // they want to add to the other user. Not checking for this would essentially
    // create a privilege escalation exploit once you had the `user.any:write` permission.
    for permission in &permissions_to_add {
        if !current_user_permissions.has_permission(*permission) {
            return Ok(response_with_reason!(
                StatusCode::FORBIDDEN,
                format!(
                    "You are not allowed to add the {} permission to other users.",
                    permission.to_name()
                )
            ));
        }
    }


    // Add the permissions to the specified user.
    mutation::user_permissions::Mutation::add_permissions_to_user_by_user_id(
        &state.database,
        requested_user_id,
        permissions_to_add,
    )
    .await
    .map_err(APIError::InternalError)?;

    // Retrieve updated list of permission for the specified user.
    let updated_permission_list =
        UserPermissions::get_from_database_by_user_id(&state.database, requested_user_id)
            .await
            .map_err(APIError::InternalError)?
            .ok_or_else(|| {
                APIError::internal_reason(
                    "BUG: Could not fetch updated permission list, user vanished from database?!",
                )
            })?;


    Ok(UserPermissionsResponse {
        permissions: updated_permission_list.to_vec_of_permission_names(),
    }
    .into_response())
}



/*
 * DELETE /{user_id}/permissions
 */

#[derive(Deserialize)]
pub struct UserPermissionRemoveRequest {
    pub permissions_to_remove: Vec<String>,
}

#[delete("/{user_id}/permissions")]
async fn remove_permissions_from_specific_user(
    user_auth: UserAuth,
    path_info: web::Path<(i32,)>,
    state: web::Data<AppState>,
    json_data: web::Json<UserPermissionRemoveRequest>,
) -> EndpointResult {
    let requested_user_id = path_info.into_inner().0;

    // Only authenticated users with the `user.any:write` permission can remove permissions
    // from other users, but not those that they themselves don't have.
    // Intended for moderation tooling.
    let current_user_permissions = user_auth
        .permissions_if_authenticated(&state.database)
        .await
        .map_err(APIError::InternalError)?
        .ok_or_else(|| APIError::NotAuthenticated)?;

    require_permission!(
        current_user_permissions,
        UserPermission::UserAnyWrite
    );


    let json_data = json_data.into_inner();

    let permissions_to_remove_result: Result<Vec<UserPermission>, &str> = json_data
        .permissions_to_remove
        .iter()
        .map(|permission_name| {
            UserPermission::from_name(permission_name.as_str()).ok_or(permission_name.as_str())
        })
        .collect::<Result<Vec<UserPermission>, &str>>();

    let permissions_to_remove = match permissions_to_remove_result {
        Ok(permissions_to_remove) => permissions_to_remove,
        Err(non_existent_permission_name) => {
            return Ok(response_with_reason!(
                StatusCode::BAD_REQUEST,
                format!("No such permission: {non_existent_permission_name}")
            ));
        }
    };

    // Validate that the current user has all of these permissions - if they don't, they can't
    // remove them from the requested user. Otherwise anyone with `user.any:write` could take
    // anything from anyone.
    for permission in &permissions_to_remove {
        if !current_user_permissions.has_permission(*permission) {
            return Ok(response_with_reason!(
                StatusCode::FORBIDDEN,
                format!(
                    "You are not allowed to remove the {} permission from other users.",
                    permission.to_name()
                )
            ));
        }
    }


    // Remove the permission from the specified user.
    mutation::user_permissions::Mutation::remove_permissions_from_user_by_user_id(
        &state.database,
        requested_user_id,
        permissions_to_remove,
    )
    .await
    .map_err(APIError::InternalError)?;

    // Retrieve updated list of permissions for the user we just modified.
    let updated_permission_list =
        UserPermissions::get_from_database_by_user_id(&state.database, requested_user_id)
            .await
            .map_err(APIError::InternalError)?
            .ok_or_else(|| {
                APIError::internal_reason(
                    "BUG: Could not fetch updated permission list, user vanished from database?!",
                )
            })?;


    Ok(UserPermissionsResponse {
        permissions: updated_permission_list.to_vec_of_permission_names(),
    }
    .into_response())
}



/*
 * Router
 */

pub fn users_router() -> Scope {
    web::scope("users")
        .service(register_user)
        .service(get_current_user_info)
        .service(get_current_user_permissions)
        .service(update_current_user_display_name)
        .service(get_specific_user_info)
        .service(get_specific_user_permissions)
        .service(update_specific_user_display_name)
        .service(add_permissions_to_specific_user)
        .service(remove_permissions_from_specific_user)
}
