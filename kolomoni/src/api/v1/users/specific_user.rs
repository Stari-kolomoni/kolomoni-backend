/*
 * GET /users/{user_id}
 */

use actix_web::{delete, get, http::StatusCode, patch, post, web, HttpResponse};
use kolomoni_auth::permissions::{Permission, UserPermissionSet};
use kolomoni_database::{
    mutation,
    query::{self, UserPermissionsExt},
};
use sea_orm::TransactionTrait;
use serde::Deserialize;
use tracing::info;
use utoipa::ToSchema;

use crate::{
    api::{
        errors::{APIError, EndpointResult},
        macros::DumbResponder,
        v1::users::{
            UserDisplayNameChangeRequest,
            UserDisplayNameChangeResponse,
            UserInfoResponse,
            UserInformation,
            UserPermissionsResponse,
        },
    },
    authentication::UserAuth,
    require_authentication,
    require_permission,
    response_with_reason,
    state::ApplicationState,
};

/// Get a specific user's information
///
/// This is a generic version of the `GET /users/me` endpoint, allowing you to see information
/// about users other than yourself.
///
/// *This endpoint requires the `users.any:read` permission.*
#[utoipa::path(
    get,
    path = "/users/{user_id}",
    tag = "users",
    params(
        ("user_id" = i32, Path, description = "ID of the user to get information about.")
    ),
    responses(
        (
            status = 200,
            description = "User information.",
            body = UserInfoResponse,
            example = json!({
                "user": {
                    "id": 1,
                    "username": "janeznovak",
                    "display_name": "Janez Novak",
                    "joined_at": "2023-06-27T20:33:53.078789Z",
                    "last_modified_at": "2023-06-27T20:34:27.217273Z",
                    "last_active_at": "2023-06-27T20:34:27.253746Z"
                }
            })
        ),
        (
            status = 401,
            description = "Missing user authentication."
        ),
        (
            status = 403,
            description = "Missing `user.any:read` permission.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "Missing permission: user.any:read." })
        ),
        (
            status = 404,
            description = "Requested user does not exist."
        ),
        (
            status = 500,
            description = "Internal server error."
        )
    ),
    security(
        ("access_token" = [])
    )
)]
#[get("/{user_id}")]
async fn get_specific_user_info(
    state: ApplicationState,
    user_auth: UserAuth,
    path_info: web::Path<(i32,)>,
) -> EndpointResult {
    // Only authenticated users with the `user.any:read` permission can access this endpoint.
    let (_, permissions) = require_authentication!(state, user_auth);
    require_permission!(permissions, Permission::UserAnyRead);


    // Return information about the requested user.
    let requested_user_id = path_info.into_inner().0;

    let optional_requested_user =
        query::UserQuery::get_user_by_id(&state.database, requested_user_id)
            .await
            .map_err(APIError::InternalError)?;

    let Some(user) = optional_requested_user else {
        return Ok(HttpResponse::NotFound().finish());
    };

    Ok(UserInfoResponse::new(user).into_response())
}



/*
 * GET /users/{user_id}/permissions
 */

/// Get a specific user's permissions
///
/// This is a generic version of the `GET /users/me/permissions` endpoint, allowing you
/// to see others' permissions.
///
/// *This endpoint requires the `users.any:read` permission.*
#[utoipa::path(
    get,
    path = "/users/{user_id}/permissions",
    tag = "users",
    params(
        ("user_id" = i32, Path, description = "ID of the user to get permissions for.")
    ),
    responses(
        (
            status = 200,
            description = "User permissions.",
            body = UserPermissionsResponse,
            example = json!({
                "permissions": [
                    "user.self:read",
                    "user.self:write",
                    "user.any:read"
                ]
            })
        ),
        (
            status = 401,
            description = "Missing user authentication."
        ),
        (
            status = 403,
            description = "Missing `user.any:read` permission.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "Missing permission: user.any:read." })
        ),
        (
            status = 404,
            description = "Requested user does not exist."
        ),
        (
            status = 500,
            description = "Internal server error."
        )
    ),
    security(
        ("access_token" = [])
    )
)]
#[get("/{user_id}/permissions")]
async fn get_specific_user_permissions(
    state: ApplicationState,
    user_auth: UserAuth,
    path_info: web::Path<(i32,)>,
) -> EndpointResult {
    // Only authenticated users with the `user.any:read` permission can access this endpoint.
    let (_, permissions) = require_authentication!(state, user_auth);
    require_permission!(permissions, Permission::UserAnyRead);


    // Get requested user's permissions.
    let requested_user_id = path_info.into_inner().0;

    let optional_user_permissions =
        query::UserPermissionQuery::get_user_permission_names_by_user_id(
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

/// Update a specific user's display name
///
/// This is generic version of the `PATCH /users/me/display_name` endpoint, allowing a user
/// with enough permissions to modify another user's display name.
///
/// *This endpoint requires the `users.any:write` permission.*
#[utoipa::path(
    patch,
    path = "/users/{user_id}/display_name",
    tag = "users",
    params(
        ("user_id" = i32, Path, description = "User ID.")
    ),
    request_body(
        content = UserDisplayNameChangeRequest,
        example = json!({
            "new_display_name": "Janez Novak Veliki"
        })
    ),
    responses(
        (
            status = 200,
            description = "User's display name changed.",
            body = UserDisplayNameChangeResponse,
            example = json!({
                "user": {
                    "id": 1,
                    "username": "janeznovak",
                    "display_name": "Janez Novak Veliki",
                    "joined_at": "2023-06-27T20:33:53.078789Z",
                    "last_modified_at": "2023-06-27T20:44:27.217273Z",
                    "last_active_at": "2023-06-27T20:34:27.253746Z"
                }
            })
        ),
        (
            status = 401,
            description = "Missing user authentication."
        ),
        (
            status = 403,
            description = "Missing `user.any:write` permission.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "Missing permission: user.any:write." })
        ),
        (
            status = 409,
            description = "User with given display name already exists.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "User with given display name already exists." })
        ),
        (
            status = 500,
            description = "Internal server error."
        )
    ),
    security(
        ("access_token" = [])
    )
)]
#[patch("/{user_id}/display_name")]
async fn update_specific_user_display_name(
    state: ApplicationState,
    user_auth: UserAuth,
    path_info: web::Path<(i32,)>,
    json_data: web::Json<UserDisplayNameChangeRequest>,
) -> EndpointResult {
    // Only authenticated users with the `user.any:write` permission can modify
    // others' display names. Intended for moderation tooling.
    let (token, permissions) = require_authentication!(state, user_auth);
    require_permission!(permissions, Permission::UserAnyWrite);


    // Disallow modifying your own account on these `/{user_id}/*` endpoints.
    let requested_user_id = path_info.into_inner().0;

    let current_user = query::UserQuery::get_user_by_username(&state.database, &token.username)
        .await
        .map_err(APIError::InternalError)?
        .ok_or_else(|| APIError::internal_reason("BUG: Current user does not exist."))?;

    if current_user.id == requested_user_id {
        return Ok(response_with_reason!(
            StatusCode::FORBIDDEN,
            "Can't modify your own account on this endpoint."
        ));
    }


    let json_data = json_data.into_inner();
    let database_transaction = state
        .database
        .begin()
        .await
        .map_err(APIError::InternalDatabaseError)?;


    // Modify requested user's display name.
    let display_name_already_exists = query::UserQuery::user_exists_by_display_name(
        &database_transaction,
        &json_data.new_display_name,
    )
    .await
    .map_err(APIError::InternalError)?;

    if display_name_already_exists {
        return Ok(response_with_reason!(
            StatusCode::CONFLICT,
            "User with given display name already exists."
        ));
    }


    // Update requested user's display name.
    mutation::UserMutation::update_display_name_by_user_id(
        &database_transaction,
        requested_user_id,
        json_data.new_display_name.clone(),
    )
    .await
    .map_err(APIError::InternalError)?;

    let updated_user = mutation::UserMutation::update_last_active_at_by_user_id(
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
        user: UserInformation::from_user_model(updated_user),
    }
    .into_response())
}



/*
 * POST /users/{user_id}/permissions
 */

/// Request containing list of permissions to add.
#[derive(Deserialize, ToSchema)]
pub struct UserPermissionsAddRequest {
    pub permissions_to_add: Vec<String>,
}


/// Add permissions to user
///
/// This endpoint allows users with enough permissions to add specific permissions to others.
/// You can add a specific permission to the requested user *only if you have that permission*.
/// If you do not, your request will be denied with a `403 Forbidden`.
///
/// *This endpoint requires the `users.any:write` permission.*
#[utoipa::path(
    post,
    path = "/users/{user_id}/permissions",
    params(
        ("user_id" = i32, Path, description = "ID of the user to add permissions to.")
    ),
    request_body(
        content = inline(UserPermissionsAddRequest),
        example = json!({
            "permissions_to_add": ["user.any:read", "user.any:write"]
        })
    ),
    responses(
        (
            status = 200,
            description = "Updated user permission list.",
            body = UserPermissionsResponse,
            example = json!({
                "permissions": [
                    "user.self:read",
                    "user.self:write",
                    "user.any:read"
                ]
            })
        ),
        (
            status = 400,
            description = "Invalid permission name.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "No such permission: \"non.existent:permission\"." })
        ),
        (
            status = 401,
            description = "Missing user authentication."
        ),
        (
            status = 403,
            description = "Not allowed to modify.",
            body = ErrorReasonResponse,
            examples(
                ("Missing user.any:write permission" = (
                    summary = "Missing user.any:write permission.",
                    value = json!({ "reason": "Missing permission: user.any:write." })
                )),
                ("Can't give permission you don't have" = (
                    summary = "Can't give permission you don't have.",
                    value = json!({ "reason": "You are not allowed to add the user.any:read permission to other users." })
                )),
                ("Can't modify yourself" = (
                    summary = "You're not allowed to modify your own account.",
                    value = json!({ "reason": "Can't modify your own account on this endpoint." })
                ))
            )
        ),
        (
            status = 500,
            description = "Internal server error."
        )
    ),
    security(
        ("access_token" = [])
    )
)]
#[post("/{user_id}/permissions")]
async fn add_permissions_to_specific_user(
    state: ApplicationState,
    user_auth: UserAuth,
    path_info: web::Path<(i32,)>,
    json_data: web::Json<UserPermissionsAddRequest>,
) -> EndpointResult {
    // Only authenticated users with the `user.any:write` permission can add permissions
    // to other users, but only if they also have the requested permission.
    // Intended for moderation tooling.
    let (current_user_token, current_user_permissions) = require_authentication!(state, user_auth);
    require_permission!(current_user_permissions, Permission::UserAnyWrite);


    let requested_user_id = path_info.into_inner().0;
    let json_data = json_data.into_inner();


    // Disallow modifying your own account on these `/{user_id}/*` endpoints.
    let current_user =
        query::UserQuery::get_user_by_username(&state.database, &current_user_token.username)
            .await
            .map_err(APIError::InternalError)?
            .ok_or_else(|| APIError::internal_reason("BUG: Current user does not exist."))?;

    if current_user.id == requested_user_id {
        return Ok(response_with_reason!(
            StatusCode::FORBIDDEN,
            "Can't modify your own account on this endpoint."
        ));
    }


    let permissions_to_add_result: Result<Vec<Permission>, &str> = json_data
        .permissions_to_add
        .iter()
        .map(|permission_name| {
            Permission::from_name(permission_name.as_str()).ok_or(permission_name.as_str())
        })
        .collect::<Result<Vec<Permission>, &str>>();

    let permissions_to_add = match permissions_to_add_result {
        Ok(permissions_to_add) => permissions_to_add,
        Err(non_existent_permission_name) => {
            return Ok(response_with_reason!(
                StatusCode::BAD_REQUEST,
                format!("No such permission: \"{non_existent_permission_name}\".")
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
                    permission.name()
                )
            ));
        }
    }


    // Add the permissions to the specified user.
    mutation::UserPermissionMutation::add_permissions_to_user_by_user_id(
        &state.database,
        requested_user_id,
        permissions_to_add,
    )
    .await
    .map_err(APIError::InternalError)?;

    // Retrieve updated list of permission for the specified user.
    let updated_permission_list =
        UserPermissionSet::get_from_database_by_user_id(&state.database, requested_user_id)
            .await
            .map_err(APIError::InternalError)?
            .ok_or_else(|| {
                APIError::internal_reason(
                    "BUG: Could not fetch updated permission list, user vanished from database?!",
                )
            })?;


    Ok(UserPermissionsResponse {
        permissions: updated_permission_list.to_permission_names(),
    }
    .into_response())
}



/*
 * DELETE /users/{user_id}/permissions
 */

/// Request to remove a list of permissions.
#[derive(Deserialize, ToSchema)]
pub struct UserPermissionsRemoveRequest {
    pub permissions_to_remove: Vec<String>,
}

/// Remove permissions from user
///
/// This endpoint allows user with enough permissions to remove specific permissions from others.
/// You can remove a specific permission from the requested user *only if you also have that permission*.
/// If you do not, your request will be denied with a `403 Forbidden`.
///
/// *This endpoint requires the `users.any:write` permission.*
#[utoipa::path(
    delete,
    path = "/users/{user_id}/permissions",
    params(
        ("user_id" = i32, Path, description = "ID of the user to remove permissions from.")
    ),
    request_body(
        content = inline(UserPermissionsRemoveRequest),
        example = json!({
            "permissions_to_remove": ["user.any:write"], 
        })
    ),
    responses(
        (
            status = 200,
            description = "Updated user permission list.",
            body = UserPermissionsResponse,
            example = json!({
                "permissions": [
                    "user.self:read",
                    "user.self:write",
                    "user.any:read"
                ]
            })
        ),
        (
            status = 400,
            description = "Invalid permission name.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "No such permission: \"non.existent:permission\"." })
        ),
        (
            status = 401,
            description = "Missing user authentication."
        ),
        (
            status = 403,
            description = "Not allowed to modify.",
            body = ErrorReasonResponse,
            examples(
                ("Missing user.any:write permission" = (
                    summary = "Missing user.any:write permission.",
                    value = json!({ "reason": "Missing permission: user.any:write." })
                )),
                ("Can't remove permission you don't have" = (
                    summary = "Can't remove permission you don't have.",
                    value = json!({ "reason": "You are not allowed to remove the user.any:read permission from other users." })
                )),
                ("Can't modify yourself" = (
                    summary = "You're not allowed to modify your own account.",
                    value = json!({ "reason": "Can't modify your own account on this endpoint." })
                ))
            )
        ),
        (
            status = 500,
            description = "Internal server error."
        )
    ),
    security(
        ("access_token" = [])
    )
)]
#[delete("/{user_id}/permissions")]
async fn remove_permissions_from_specific_user(
    state: ApplicationState,
    user_auth: UserAuth,
    path_info: web::Path<(i32,)>,
    json_data: web::Json<UserPermissionsRemoveRequest>,
) -> EndpointResult {
    // Only authenticated users with the `user.any:write` permission can remove permissions
    // from other users, but not those that they themselves don't have.
    // Intended for moderation tooling.
    let (current_user_token, current_user_permissions) = require_authentication!(state, user_auth);
    require_permission!(current_user_permissions, Permission::UserAnyWrite);


    let requested_user_id = path_info.into_inner().0;
    let json_data = json_data.into_inner();


    // Disallow modifying your own account on these `/{user_id}/*` endpoints.
    let current_user =
        query::UserQuery::get_user_by_username(&state.database, &current_user_token.username)
            .await
            .map_err(APIError::InternalError)?
            .ok_or_else(|| APIError::internal_reason("BUG: Current user does not exist."))?;

    if current_user.id == requested_user_id {
        return Ok(response_with_reason!(
            StatusCode::FORBIDDEN,
            "Can't modify your own account on this endpoint."
        ));
    }


    let permissions_to_remove_result: Result<Vec<Permission>, &str> = json_data
        .permissions_to_remove
        .iter()
        .map(|permission_name| {
            Permission::from_name(permission_name.as_str()).ok_or(permission_name.as_str())
        })
        .collect::<Result<Vec<Permission>, &str>>();

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
                    permission.name()
                )
            ));
        }
    }


    // Remove the permission from the specified user.
    mutation::UserPermissionMutation::remove_permissions_from_user_by_user_id(
        &state.database,
        requested_user_id,
        permissions_to_remove,
    )
    .await
    .map_err(APIError::InternalError)?;

    // Retrieve updated list of permissions for the user we just modified.
    let updated_permission_list =
        UserPermissionSet::get_from_database_by_user_id(&state.database, requested_user_id)
            .await
            .map_err(APIError::InternalError)?
            .ok_or_else(|| {
                APIError::internal_reason(
                    "BUG: Could not fetch updated permission list, user vanished from database?!",
                )
            })?;


    Ok(UserPermissionsResponse {
        permissions: updated_permission_list.to_permission_names(),
    }
    .into_response())
}
