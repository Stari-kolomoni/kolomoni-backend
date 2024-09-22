use std::collections::HashSet;

use actix_web::{delete, get, http::StatusCode, patch, post, web, HttpResponse};
use kolomoni_auth::{Permission, Role, RoleSet};
use kolomoni_core::{
    api_models::{UserDisplayNameChangeRequest, UserDisplayNameChangeResponse},
    id::UserId,
};
use kolomoni_database::entities;
use serde::Deserialize;
use sqlx::{types::Uuid, Acquire};
use tracing::info;
use utoipa::ToSchema;

use crate::{
    api::{
        errors::{APIError, EndpointResult},
        macros::ContextlessResponder,
        openapi,
        traits::IntoApiModel,
        v1::users::{UserInfoResponse, UserPermissionsResponse, UserRolesResponse},
    },
    authentication::UserAuthenticationExtractor,
    json_error_response_with_reason,
    obtain_database_connection,
    require_authentication,
    require_permission,
    require_permission_with_optional_authentication,
    state::ApplicationState,
};



/// Get a user's information
///
/// This is an expanded version of the `GET /users/me` endpoint,
/// allowing you to see information about users other than yourself.
///
/// # Authentication
/// Authentication is *not required* on this endpoint due to a blanket grant of
/// the `users.any:read` permission to unauthenticated users.
#[utoipa::path(
    get,
    path = "/users/{user_id}",
    tag = "users",
    params(
        (
            "user_id" = Uuid,
            Path,
            description = "ID of the user to get information about."
        )
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
            status = 404,
            description = "Requested user does not exist."
        ),
        openapi::FailedAuthenticationResponses<openapi::RequiresUserAnyRead>,
        openapi::InternalServerErrorResponse,
    )
)]
#[get("/{user_id}")]
async fn get_specific_user_info(
    state: ApplicationState,
    authentication_extractor: UserAuthenticationExtractor,
    path_info: web::Path<(Uuid,)>,
) -> EndpointResult {
    let mut database_connection = obtain_database_connection!(state);


    // Users don't need to authenticate due to a
    // blanket permission grant for `user.any:read`.
    // This will also work if we remove the blanket grant
    // in the future - it will fall back to requiring authentication
    // AND the `user.any:read` permission.
    require_permission_with_optional_authentication!(
        &mut database_connection,
        authentication_extractor,
        Permission::UserAnyRead
    );


    // Return information about the requested user.
    let requested_user_id = path_info.into_inner().0;


    let user_info_if_they_exist = entities::UserQuery::get_user_by_id(
        &mut database_connection,
        UserId::new(requested_user_id),
    )
    .await?;


    let Some(user_info) = user_info_if_they_exist else {
        return Ok(HttpResponse::NotFound().finish());
    };

    Ok(UserInfoResponse {
        user: user_info.into_api_model(),
    }
    .into_response())
}



/// Get a user's roles
///
/// # Authentication
/// Authentication is *not required* on this endpoint due to a blanket grant of
/// the `users.any:read` permission to unauthenticated users.
#[utoipa::path(
    get,
    path = "/users/{user_id}/roles",
    tag = "users",
    params(
        (
            "user_id" = Uuid,
            Path,
            description = "ID of the user to query roles for."
        )
    ),
    responses(
        (
            status = 200,
            description = "User role list.",
            body = UserRolesResponse
        ),
        (
            status = 404,
            description = "No user with provided ID."
        ),
        openapi::FailedAuthenticationResponses<openapi::RequiresUserAnyRead>,
        openapi::InternalServerErrorResponse,
    )
)]
#[get("/{user_id}/roles")]
pub async fn get_specific_user_roles(
    state: ApplicationState,
    authentication_extractor: UserAuthenticationExtractor,
    path_info: web::Path<(Uuid,)>,
) -> EndpointResult {
    let mut database_connection = obtain_database_connection!(state);


    // Users don't need to authenticate due to a
    // blanket permission grant for `user.any:read`.
    // This will also work if we remove the blanket grant
    // in the future - it will fall back to requiring authentication
    // AND the `user.any:read` permission.
    require_permission_with_optional_authentication!(
        &mut database_connection,
        authentication_extractor,
        Permission::UserAnyRead
    );


    let target_user_id = UserId::new(path_info.into_inner().0);


    let target_user_exists =
        entities::UserQuery::exists_by_id(&mut database_connection, target_user_id).await?;

    if !target_user_exists {
        return Err(APIError::not_found());
    }


    let target_user_role_set =
        entities::UserRoleQuery::roles_for_user(&mut database_connection, target_user_id).await?;


    let target_user_role_names = target_user_role_set.role_names();

    Ok(UserRolesResponse {
        role_names: target_user_role_names,
    }
    .into_response())
}



/// Get a user's effective permissions
///
/// Returns a list of effective permissions.
/// The effective permission list depends on permissions that each of the user's roles provide.
///
/// This is a generic version of the `GET /users/me/permissions` endpoint,
/// allowing you to see others' permissions.
///
/// # Authentication
/// This endpoint requires authentication and the `users.any:read` permission.
#[utoipa::path(
    get,
    path = "/users/{user_id}/permissions",
    tag = "users",
    params(
        (
            "user_id" = Uuid,
            Path,
            description = "ID of the user to get effective permissions for."
        )
    ),
    responses(
        (
            status = 200,
            description = "User permissions.",
            body = UserPermissionsResponse,
        ),
        (
            status = 404,
            description = "Requested user does not exist."
        ),
        openapi::FailedAuthenticationResponses<openapi::RequiresUserAnyRead>,
        openapi::InternalServerErrorResponse,
    ),
    security(
        ("access_token" = [])
    )
)]
#[get("/{user_id}/permissions")]
async fn get_specific_user_effective_permissions(
    state: ApplicationState,
    authentication_extractor: UserAuthenticationExtractor,
    path_info: web::Path<(Uuid,)>,
) -> EndpointResult {
    let mut database_connection = obtain_database_connection!(state);


    // Only authenticated users with the `user.any:read` permission can access this endpoint.
    let authenticated_user = require_authentication!(authentication_extractor);
    require_permission!(
        &mut database_connection,
        authenticated_user,
        Permission::UserAnyRead
    );


    // Get requested user's permissions.
    let target_user_id = UserId::new(path_info.into_inner().0);


    let target_user_exists =
        entities::UserQuery::exists_by_id(&mut database_connection, target_user_id).await?;

    if !target_user_exists {
        return Ok(HttpResponse::NotFound().finish());
    }


    let target_user_permission_set = entities::UserRoleQuery::transitive_permissions_for_user(
        &mut database_connection,
        target_user_id,
    )
    .await?;

    let permission_names = target_user_permission_set
        .permission_names()
        .into_iter()
        .map(|name| name.to_string())
        .collect();


    Ok(UserPermissionsResponse {
        permissions: permission_names,
    }
    .into_response())
}

// TODO Continue from here.


/// Update a user's display name
///
/// This is generic version of the `PATCH /users/me/display_name` endpoint,
/// allowing a user with enough permissions to modify another user's display name.
///
/// # Restrictions
/// You can not modify your own roles on this endpoint.
///
/// # Authentication
/// This endpoint requires authentication and the `users.any:write` permission.
#[utoipa::path(
    patch,
    path = "/users/{user_id}/display_name",
    tag = "users",
    params(
        (
            "user_id" = Uuid,
            Path,
            description = "User ID."
        )
    ),
    request_body(
        content = UserDisplayNameChangeRequest
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
            status = 404,
            description = "User with the given ID does not exist.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "Resource not found: no such user." })
        ),
        (
            status = 409,
            description = "User with given display name already exists.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "User with given display name already exists." })
        ),
        openapi::MissingOrInvalidJsonRequestBodyResponse,
        openapi::FailedAuthenticationResponses<openapi::RequiresUserAnyWrite>,
        openapi::InternalServerErrorResponse,
    ),
    security(
        ("access_token" = [])
    )
)]
#[patch("/{user_id}/display_name")]
async fn update_specific_user_display_name(
    state: ApplicationState,
    authentication_extractor: UserAuthenticationExtractor,
    path_info: web::Path<(Uuid,)>,
    json_data: web::Json<UserDisplayNameChangeRequest>,
) -> EndpointResult {
    let mut database_connection = obtain_database_connection!(state);
    let mut transaction = database_connection.begin().await?;


    // Only authenticated users with the `user.any:write` permission can modify
    // others' display names. Intended for moderation tooling.
    let authenticated_user = require_authentication!(authentication_extractor);
    require_permission!(
        &mut transaction,
        authenticated_user,
        Permission::UserAnyWrite
    );


    let authenticated_user_id = authenticated_user.user_id();
    let target_user_id = UserId::new(path_info.into_inner().0);


    // Disallow modifying your own account on these `/{user_id}/*` endpoints.
    if authenticated_user_id == target_user_id {
        return Ok(json_error_response_with_reason!(
            StatusCode::FORBIDDEN,
            "Can't modify your own account on this endpoint."
        ));
    }


    let target_user_exists =
        entities::UserQuery::exists_by_id(&mut transaction, target_user_id).await?;

    if !target_user_exists {
        return Err(APIError::not_found_with_reason("no such user"));
    }


    let change_request_data = json_data.into_inner();


    // Modify requested user's display name.
    let display_name_already_exists = entities::UserQuery::exists_by_display_name(
        &mut transaction,
        &change_request_data.new_display_name,
    )
    .await?;

    if display_name_already_exists {
        return Ok(json_error_response_with_reason!(
            StatusCode::CONFLICT,
            "User with given display name already exists."
        ));
    }


    // Update requested user's display name.
    let updated_user = entities::UserMutation::change_display_name_by_user_id(
        &mut transaction,
        target_user_id,
        &change_request_data.new_display_name,
    )
    .await?;


    transaction.commit().await?;

    info!(
        operator_id = %authenticated_user_id,
        target_user_id = %target_user_id,
        new_display_name = %change_request_data.new_display_name,
        "User has updated another user's display name."
    );


    Ok(UserDisplayNameChangeResponse {
        user: updated_user.into_api_model(),
    }
    .into_response())
}




#[derive(Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(serde::Serialize))]
#[schema(
    example = json!({
        "roles_to_add": ["administrator"]
    })
)]
pub struct UserRoleAddRequest {
    pub roles_to_add: Vec<String>,
}


/// Add roles to a user
///
/// This endpoint allows a user with enough permissions to add roles to another user.
///
/// # Restrictions
/// You can not modify your own roles on this endpoint.
///
/// # Authentication
/// This endpoint requires authentication and the `users.any:write` permission.
/// Additionally, you can not give out a role you do not have yourself -- trying to do
/// so will fail with `403 Forbidden`.
#[utoipa::path(
    post,
    path = "/users/{user_id}/roles",
    tag = "users",
    params(
        (
            "user_id" = Uuid,
            Path,
            description = "ID of the user to add roles to."
        )
    ),
    request_body(
        content = UserRoleAddRequest
    ),
    responses(
        (
            status = 200,
            description = "Updated user role list.",
            body = UserRolesResponse
        ),
        (
            status = 400,
            description = "Invalid role name.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "No such role: \"non-existent-role-name\"." })
        ),
        (
            status = 403,
            description = "Not allowed to modify roles.",
            body = ErrorReasonResponse,
            examples(
                ("Can't give out roles you don't have" = (
                    summary = "Can't give out roles you don't have.",
                    value = json!({ "reason": "You cannot give out roles you do not have (missing role: administrator)." })
                )),
                ("Can't modify yourself" = (
                    summary = "You're not allowed to modify your own account.",
                    value = json!({ "reason": "Can't modify your own account on this endpoint." })
                ))
            )
        ),
        (
            status = 404,
            description = "The specified user does not exist.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "The specified user does not exist." })
        ),
        openapi::MissingOrInvalidJsonRequestBodyResponse,
        openapi::FailedAuthenticationResponses<openapi::RequiresUserAnyWrite>,
        openapi::InternalServerErrorResponse,
    ),
    security(
        ("access_token" = [])
    )
)]
#[post("/{user_id}/roles")]
pub async fn add_roles_to_specific_user(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    path_info: web::Path<(Uuid,)>,
    json_data: web::Json<UserRoleAddRequest>,
) -> EndpointResult {
    let mut database_connection = obtain_database_connection!(state);
    let mut transaction = database_connection.begin().await?;


    // Only authenticated users with the `user.any:write` permission can add roles
    // to other users, but only if they also have that role.
    // Intended for moderation tooling.
    let authenticated_user = require_authentication!(authentication);
    let authenticated_user_roles = authenticated_user.fetch_roles(&mut transaction).await?;
    let authenticated_user_permissions = authenticated_user_roles.granted_permission_set();

    require_permission!(
        authenticated_user_permissions,
        Permission::UserAnyWrite
    );


    let authenticated_user_id = authenticated_user.user_id();
    let target_user_id = UserId::new(path_info.into_inner().0);

    let request_data = json_data.into_inner();


    // Disallow modifying your own user account on this endpoint.
    if authenticated_user_id == target_user_id {
        return Ok(json_error_response_with_reason!(
            StatusCode::FORBIDDEN,
            "Can't modify your own account on this endpoint."
        ));
    }


    let mut roles_to_add_to_user = HashSet::with_capacity(request_data.roles_to_add.len());

    for raw_role_name in request_data.roles_to_add {
        let Some(role) = Role::from_name(&raw_role_name) else {
            return Ok(json_error_response_with_reason!(
                StatusCode::BAD_REQUEST,
                format!("{} is an invalid role name", raw_role_name)
            ));
        };

        roles_to_add_to_user.insert(role);
    }

    let roles_to_add_to_user = RoleSet::from_role_hash_set(roles_to_add_to_user);


    // Validate that the authenticated user has all of the roles
    // they wish to assign to other users. Not checking for this would
    // be dangerous as it would essentially allow for privilege escalation.
    for role in roles_to_add_to_user.roles() {
        if !authenticated_user_roles.has_role(role) {
            return Ok(json_error_response_with_reason!(
                StatusCode::FORBIDDEN,
                format!(
                    "You cannot give out roles you do not have (missing role: {}).",
                    role.name()
                )
            ));
        }
    }



    let user_exists = entities::UserQuery::exists_by_id(&mut transaction, target_user_id).await?;

    if !user_exists {
        return Err(APIError::not_found_with_reason(
            "The specified user does not exist.",
        ));
    }


    let full_updated_user_role_set = entities::UserRoleMutation::add_roles_to_user(
        &mut transaction,
        target_user_id,
        roles_to_add_to_user,
    )
    .await?;


    transaction.commit().await?;


    Ok(UserRolesResponse {
        role_names: full_updated_user_role_set.role_names(),
    }
    .into_response())
}




#[derive(Deserialize, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(serde::Serialize))]
#[schema(
    example = json!({
        "roles_to_remove": ["administrator"]
    })
)]
pub struct UserRoleRemoveRequest {
    pub roles_to_remove: Vec<String>,
}


/// Removes roles from a user
///
/// This endpoint allows a user with enough permission to remove roles from another user.
///
/// # Restrictions
/// You can not modify your own roles on this endpoint.
///
/// # Authentication
/// This endpoint requires authentication and the `users.any:write` permission.
/// Additionally, you can not remove a role you do not have yourself -- trying to do
/// so will fail with `403 Forbidden`.
#[utoipa::path(
    delete,
    path = "/users/{user_id}/roles",
    tag = "users",
    params(
        (
            "user_id" = Uuid,
            Path,
            description = "ID of the user to remove roles from."
        )
    ),
    request_body(
        content = UserRoleRemoveRequest
    ),
    responses(
        (
            status = 200,
            description = "Updated user role list.",
            body = UserRolesResponse
        ),
        (
            status = 400,
            description = "Invalid role name.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "No such role: \"non-existent-role-name\"." })
        ),
        (
            status = 403,
            description = "Not allowed to modify roles.",
            body = ErrorReasonResponse,
            examples(
                ("Can't remove others' roles you don't have" = (
                    summary = "Can't give out roles you don't have.",
                    value = json!({ "reason": "You cannot remove others' roles which you do not have (missing role: administrator)." })
                )),
                ("Can't modify yourself" = (
                    summary = "You're not allowed to modify your own account.",
                    value = json!({ "reason": "Can't modify your own account on this endpoint." })
                ))
            )
        ),
        (
            status = 404,
            description = "The specified user does not exist.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "The specified user does not exist." })
        ),
        openapi::MissingOrInvalidJsonRequestBodyResponse,
        openapi::FailedAuthenticationResponses<openapi::RequiresUserAnyWrite>,
        openapi::InternalServerErrorResponse,
    ),
    security(
        ("access_token" = [])
    )
)]
#[delete("/{user_id}/roles")]
pub async fn remove_roles_from_specific_user(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    path_info: web::Path<(Uuid,)>,
    json_data: web::Json<UserRoleRemoveRequest>,
) -> EndpointResult {
    let mut database_connection = obtain_database_connection!(state);
    let mut transaction = database_connection.begin().await?;


    // Only authenticated users with the `user.any:write` permission can remove roles
    // from other users, but only if they also have that role.
    // Intended for moderation tooling.
    let authenticated_user = require_authentication!(authentication);
    let authenticated_user_roles = authenticated_user.fetch_roles(&mut transaction).await?;
    let authenticated_user_permissions = authenticated_user_roles.granted_permission_set();

    require_permission!(
        authenticated_user_permissions,
        Permission::UserAnyWrite
    );


    let authenticated_user_id = authenticated_user.user_id();
    let target_user_id = UserId::new(path_info.into_inner().0);

    let request_data = json_data.into_inner();


    // Disallow modifying your own user account on this endpoint.
    if authenticated_user_id == target_user_id {
        return Ok(json_error_response_with_reason!(
            StatusCode::FORBIDDEN,
            "Can't modify your own account on this endpoint."
        ));
    }


    let mut roles_to_remove_from_user = HashSet::with_capacity(request_data.roles_to_remove.len());

    for raw_role_name in request_data.roles_to_remove {
        let Some(role) = Role::from_name(&raw_role_name) else {
            return Ok(json_error_response_with_reason!(
                StatusCode::BAD_REQUEST,
                format!("{} is an invalid role name", raw_role_name)
            ));
        };

        roles_to_remove_from_user.insert(role);
    }

    let roles_to_remove_from_user = RoleSet::from_role_hash_set(roles_to_remove_from_user);


    // Validate that the authenticated user (caller) has all of the roles
    // they wish to remove from the target user. Not checking for this would
    // be dangerous as it would essentially allow for privilege de-escalation.
    for role in roles_to_remove_from_user.roles() {
        if !authenticated_user_roles.has_role(role) {
            return Ok(json_error_response_with_reason!(
                StatusCode::FORBIDDEN,
                format!(
                    "You cannot remove others' roles which you do not have (missing role: {}).",
                    role.name()
                )
            ));
        }
    }


    let user_exists = entities::UserQuery::exists_by_id(&mut transaction, target_user_id).await?;

    if !user_exists {
        return Err(APIError::not_found_with_reason(
            "The specified user does not exist.",
        ));
    }


    let full_updated_user_role_set = entities::UserRoleMutation::remove_roles_from_user(
        &mut transaction,
        target_user_id,
        roles_to_remove_from_user,
    )
    .await?;


    Ok(UserRolesResponse {
        role_names: full_updated_user_role_set.role_names(),
    }
    .into_response())
}
