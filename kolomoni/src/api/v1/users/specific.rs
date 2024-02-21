use actix_web::{delete, get, http::StatusCode, patch, post, web, HttpResponse};
use kolomoni_auth::{Permission, Role};
use kolomoni_database::{
    mutation,
    query::{self, UserQuery, UserRoleQuery},
};
use sea_orm::TransactionTrait;
use serde::Deserialize;
use tracing::info;
use utoipa::ToSchema;

use crate::{
    api::{
        errors::{APIError, EndpointResult},
        macros::ContextlessResponder,
        openapi,
        v1::users::{
            UserDisplayNameChangeRequest,
            UserDisplayNameChangeResponse,
            UserInfoResponse,
            UserInformation,
            UserPermissionsResponse,
            UserRolesResponse,
        },
    },
    authentication::UserAuthenticationExtractor,
    error_response_with_reason,
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
#[get("/{user_id}")]
async fn get_specific_user_info(
    state: ApplicationState,
    authentication_extractor: UserAuthenticationExtractor,
    path_info: web::Path<(i32,)>,
) -> EndpointResult {
    // Users don't need to authenticate due to a
    // blanket permission grant for `user.any:read`.
    // This will also work if we remove the blanket grant
    // in the future - it will fall back to requiring authentication
    // AND the `user.any:read` permission.
    require_permission_with_optional_authentication!(
        state,
        authentication_extractor,
        Permission::UserAnyRead
    );


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
            "user_id" = i32,
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
    ),
    security(
        ("access_token" = [])
    )
)]
#[get("/{user_id}/roles")]
pub async fn get_specific_user_roles(
    state: ApplicationState,
    authentication_extractor: UserAuthenticationExtractor,
    path_info: web::Path<(i32,)>,
) -> EndpointResult {
    // Users don't need to authenticate due to a
    // blanket permission grant for `user.any:read`.
    // This will also work if we remove the blanket grant
    // in the future - it will fall back to requiring authentication
    // AND the `user.any:read` permission.
    require_permission_with_optional_authentication!(
        state,
        authentication_extractor,
        Permission::UserAnyRead
    );

    let target_user_id = path_info.into_inner().0;
    let target_user_exists = UserQuery::user_exists_by_user_id(&state.database, target_user_id)
        .await
        .map_err(APIError::InternalError)?;

    if !target_user_exists {
        return Err(APIError::not_found());
    }


    let target_user_roles = UserRoleQuery::user_roles(&state.database, target_user_id)
        .await
        .map_err(APIError::InternalError)?;

    let target_user_role_names = target_user_roles
        .into_roles()
        .into_iter()
        .map(|role| role.name().to_string())
        .collect();


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
            "user_id" = i32,
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
    path_info: web::Path<(i32,)>,
) -> EndpointResult {
    // Only authenticated users with the `user.any:read` permission can access this endpoint.
    let authenticated_user = require_authentication!(authentication_extractor);
    require_permission!(state, authenticated_user, Permission::UserAnyRead);


    // Get requested user's permissions.
    let target_user_id = path_info.into_inner().0;

    let target_user_exists =
        query::UserQuery::user_exists_by_user_id(&state.database, target_user_id)
            .await
            .map_err(APIError::InternalError)?;

    if !target_user_exists {
        return Ok(HttpResponse::NotFound().finish());
    }


    let target_user_permission_set = query::UserRoleQuery::effective_user_permissions_from_user_id(
        &state.database,
        target_user_id,
    )
    .await
    .map_err(APIError::InternalError)?;

    let permission_names = target_user_permission_set
        .into_permissions()
        .into_iter()
        .map(|permission| permission.name().to_string())
        .collect();


    Ok(UserPermissionsResponse {
        permissions: permission_names,
    }
    .into_response())
}



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
            "user_id" = i32,
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
    path_info: web::Path<(i32,)>,
    json_data: web::Json<UserDisplayNameChangeRequest>,
) -> EndpointResult {
    // Only authenticated users with the `user.any:write` permission can modify
    // others' display names. Intended for moderation tooling.
    let authenticated_user = require_authentication!(authentication_extractor);
    let authenticated_user_id = authenticated_user.user_id();
    require_permission!(
        state,
        authenticated_user,
        Permission::UserAnyWrite
    );


    // Disallow modifying your own account on these `/{user_id}/*` endpoints.
    let target_user_id = path_info.into_inner().0;

    if authenticated_user_id == target_user_id {
        return Ok(error_response_with_reason!(
            StatusCode::FORBIDDEN,
            "Can't modify your own account on this endpoint."
        ));
    }


    let target_user_exists =
        query::UserQuery::user_exists_by_user_id(&state.database, target_user_id)
            .await
            .map_err(APIError::InternalError)?;

    if !target_user_exists {
        return Err(APIError::not_found_with_reason("no such user"));
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
        return Ok(error_response_with_reason!(
            StatusCode::CONFLICT,
            "User with given display name already exists."
        ));
    }


    // Update requested user's display name.
    mutation::UserMutation::update_display_name_by_user_id(
        &database_transaction,
        target_user_id,
        json_data.new_display_name.clone(),
    )
    .await
    .map_err(APIError::InternalError)?;

    let updated_user = mutation::UserMutation::update_last_active_at_by_user_id(
        &database_transaction,
        target_user_id,
        None,
    )
    .await
    .map_err(APIError::InternalError)?;

    database_transaction
        .commit()
        .await
        .map_err(APIError::InternalDatabaseError)?;


    info!(
        operator_id = authenticated_user_id,
        target_user_id = target_user_id,
        new_display_name = json_data.new_display_name,
        "User has updated another user's display name."
    );

    Ok(UserDisplayNameChangeResponse {
        user: UserInformation::from_user_model(updated_user),
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
            "user_id" = i32,
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
    path_info: web::Path<(i32,)>,
    json_data: web::Json<UserRoleAddRequest>,
) -> EndpointResult {
    // Only authenticated users with the `user.any:write` permission can add roles
    // to other users, but only if they also have that role.
    // Intended for moderation tooling.
    let authenticated_user = require_authentication!(authentication);
    let authenticated_user_id = authenticated_user.user_id();
    let authenticated_user_roles = authenticated_user
        .roles(&state.database)
        .await
        .map_err(APIError::InternalError)?;

    require_permission!(
        state,
        authenticated_user,
        Permission::UserAnyWrite
    );


    let target_user_id = path_info.into_inner().0;
    let request_data = json_data.into_inner();


    // Disallow modifying your own user account on this endpoint.
    if authenticated_user_id == target_user_id {
        return Ok(error_response_with_reason!(
            StatusCode::FORBIDDEN,
            "Can't modify your own account on this endpoint."
        ));
    }


    let parsed_roles_to_add_result = request_data
        .roles_to_add
        .into_iter()
        .map(|role_name| {
            Role::from_name(&role_name).ok_or_else(|| format!("No such role: \"{role_name}\"."))
        })
        .collect::<Result<Vec<_>, _>>();

    let roles_to_add = match parsed_roles_to_add_result {
        Ok(roles) => roles,
        Err(error_reason) => {
            return Ok(error_response_with_reason!(
                StatusCode::BAD_REQUEST,
                error_reason
            ));
        }
    };


    // Validate that the authenticated user has all of the roles
    // they wish to assign to other users. Not checking for this would
    // be dangerous as it would essentially allow for privilege escalation.
    for role in roles_to_add.iter() {
        if !authenticated_user_roles.has_role(role) {
            return Ok(error_response_with_reason!(
                StatusCode::FORBIDDEN,
                format!(
                    "You cannot give out roles you do not have (missing role: {}).",
                    role.name()
                )
            ));
        }
    }


    let user_exists = query::UserQuery::user_exists_by_user_id(&state.database, target_user_id)
        .await
        .map_err(APIError::InternalError)?;

    if !user_exists {
        return Err(APIError::not_found_with_reason(
            "The specified user does not exist.",
        ));
    }


    mutation::UserRoleMutation::add_roles_to_user(&state.database, target_user_id, &roles_to_add)
        .await
        .map_err(APIError::InternalError)?;

    let updated_role_set = query::UserRoleQuery::user_roles(&state.database, target_user_id)
        .await
        .map_err(APIError::InternalError)?;


    Ok(UserRolesResponse {
        role_names: updated_role_set.role_names(),
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
            "user_id" = i32,
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
    path_info: web::Path<(i32,)>,
    json_data: web::Json<UserRoleRemoveRequest>,
) -> EndpointResult {
    // Only authenticated users with the `user.any:write` permission can remove roles
    // from other users, but only if they also have that role.
    // Intended for moderation tooling.
    let authenticated_user = require_authentication!(authentication);
    let authenticated_user_id = authenticated_user.user_id();
    let authenticated_user_roles = authenticated_user
        .roles(&state.database)
        .await
        .map_err(APIError::InternalError)?;

    require_permission!(
        state,
        authenticated_user,
        Permission::UserAnyWrite
    );


    let target_user_id = path_info.into_inner().0;
    let request_data = json_data.into_inner();


    // Disallow modifying your own user account on this endpoint.
    if authenticated_user_id == target_user_id {
        return Ok(error_response_with_reason!(
            StatusCode::FORBIDDEN,
            "Can't modify your own account on this endpoint."
        ));
    }


    let parsed_roles_to_remove_result = request_data
        .roles_to_remove
        .into_iter()
        .map(|role_name| {
            Role::from_name(&role_name).ok_or_else(|| format!("No such role: \"{role_name}\"."))
        })
        .collect::<Result<Vec<_>, _>>();

    let roles_to_remove = match parsed_roles_to_remove_result {
        Ok(roles) => roles,
        Err(error_reason) => {
            return Ok(error_response_with_reason!(
                StatusCode::BAD_REQUEST,
                error_reason
            ));
        }
    };


    // Validate that the authenticated user (caller) has all of the roles
    // they wish to remove from the target user. Not checking for this would
    // be dangerous as it would essentially allow for privilege de-escalation.
    for role in roles_to_remove.iter() {
        if !authenticated_user_roles.has_role(role) {
            return Ok(error_response_with_reason!(
                StatusCode::FORBIDDEN,
                format!(
                    "You cannot remove others' roles which you do not have (missing role: {}).",
                    role.name()
                )
            ));
        }
    }


    let user_exists = query::UserQuery::user_exists_by_user_id(&state.database, target_user_id)
        .await
        .map_err(APIError::InternalError)?;

    if !user_exists {
        return Err(APIError::not_found_with_reason(
            "The specified user does not exist.",
        ));
    }


    mutation::UserRoleMutation::remove_roles_from_user(
        &state.database,
        target_user_id,
        &roles_to_remove,
    )
    .await
    .map_err(APIError::InternalError)?;

    let updated_role_set = query::UserRoleQuery::user_roles(&state.database, target_user_id)
        .await
        .map_err(APIError::InternalError)?;


    Ok(UserRolesResponse {
        role_names: updated_role_set.role_names(),
    }
    .into_response())
}
