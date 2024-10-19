use std::collections::HashSet;

use actix_web::{delete, get, patch, post, web};
use kolomoni_core::api_models::UsersErrorReason;
use kolomoni_core::permissions::Permission;
use kolomoni_core::roles::{Role, RoleSet};
use kolomoni_core::{
    api_models::{
        UserDisplayNameChangeRequest,
        UserDisplayNameChangeResponse,
        UserInfoResponse,
        UserPermissionsResponse,
        UserRoleAddRequest,
        UserRoleRemoveRequest,
        UserRolesResponse,
    },
    ids::UserId,
};
use kolomoni_database::entities;
use tracing::info;

use crate::{
    api::{
        errors::{EndpointResponseBuilder, EndpointResult},
        openapi::{
            self,
            response::{requires, AsErrorReason},
        },
        traits::IntoApiModel,
        v1::dictionary::parse_uuid,
    },
    authentication::UserAuthenticationExtractor,
    declare_openapi_error_reason_response,
    require_permission_in_set,
    require_permission_with_optional_authentication,
    require_user_authentication,
    require_user_authentication_and_permissions,
    state::ApplicationState,
};



declare_openapi_error_reason_response!(
    pub struct SpecificUserNotFound {
        description => "User not found.",
        reason => UsersErrorReason::user_not_found()
    }
);


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
            "user_id" = String,
            Path,
            format = Uuid,
            description = "UUID of the user to get information about."
        )
    ),
    responses(
        (
            status = 200,
            description = "User information.",
            body = UserInfoResponse,
            example = json!({
                "user": {
                    "id": "01921e6d-a0cb-724c-b8aa-31f5b4c78267",
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
            response = inline(AsErrorReason<SpecificUserNotFound>)
        ),
        openapi::response::UuidUrlParameterError,
        openapi::response::MissingPermissions<requires::UserAnyRead, 1>,
        openapi::response::InternalServerError,
    )
)]
#[get("/{user_id}")]
async fn get_specific_user_info(
    state: ApplicationState,
    authentication_extractor: UserAuthenticationExtractor,
    path_info: web::Path<(String,)>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;

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
    let requested_user_id = parse_uuid::<UserId>(path_info.into_inner().0)?;


    let user_info_if_they_exist =
        entities::UserQuery::get_user_by_id(&mut database_connection, requested_user_id).await?;

    let Some(user_info) = user_info_if_they_exist else {
        return EndpointResponseBuilder::not_found()
            .with_error_reason(UsersErrorReason::user_not_found())
            .build();
    };


    EndpointResponseBuilder::ok()
        .with_json_body(UserInfoResponse {
            user: user_info.into_api_model(),
        })
        .build()
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
            "user_id" = String,
            Path,
            format = Uuid,
            description = "UUID of the user to query roles for."
        )
    ),
    responses(
        (
            status = 200,
            description = "User's role list.",
            body = UserRolesResponse
        ),
        (
            status = 404,
            response = inline(AsErrorReason<SpecificUserNotFound>)
        ),
        openapi::response::UuidUrlParameterError,
        openapi::response::MissingPermissions<requires::UserAnyRead, 1>,
        openapi::response::InternalServerError,
    )
)]
#[get("/{user_id}/roles")]
pub async fn get_specific_user_roles(
    state: ApplicationState,
    authentication_extractor: UserAuthenticationExtractor,
    path_info: web::Path<(String,)>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;
    let mut transaction = database_connection.transaction().begin().await?;


    // Users don't need to authenticate due to a
    // blanket permission grant for `user.any:read`.
    // This will also work if we remove the blanket grant
    // in the future - it will fall back to requiring authentication
    // AND the `user.any:read` permission.
    require_permission_with_optional_authentication!(
        &mut transaction,
        authentication_extractor,
        Permission::UserAnyRead
    );


    let requested_user_id = parse_uuid::<UserId>(path_info.into_inner().0)?;


    let target_user_exists =
        entities::UserQuery::exists_by_id(&mut transaction, requested_user_id).await?;

    if !target_user_exists {
        return EndpointResponseBuilder::not_found()
            .with_error_reason(UsersErrorReason::user_not_found())
            .build();
    }


    let target_user_role_set =
        entities::UserRoleQuery::roles_for_user(&mut transaction, requested_user_id).await?;


    EndpointResponseBuilder::ok()
        .with_json_body(UserRolesResponse {
            role_names: target_user_role_set.role_names_cow(),
        })
        .build()
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
            "user_id" = String,
            Path,
            format = Uuid,
            description = "UUID of the user to get effective permissions for."
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
            response = inline(AsErrorReason<SpecificUserNotFound>)
        ),
        openapi::response::UuidUrlParameterError,
        openapi::response::MissingPermissions<requires::UserAnyRead, 1>,
        openapi::response::InternalServerError,
    ),
    security(
        ("access_token" = [])
    )
)]
#[get("/{user_id}/permissions")]
async fn get_specific_user_effective_permissions(
    state: ApplicationState,
    authentication_extractor: UserAuthenticationExtractor,
    path_info: web::Path<(String,)>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;


    // To access this endpoint, the user:
    // - MUST provide an authentication token, and
    // - MUST have the `user.self:read` permission.
    require_user_authentication_and_permissions!(
        &mut database_connection,
        authentication_extractor,
        Permission::UserAnyRead
    );


    // Get requested user's permissions.
    let requested_user_id = parse_uuid::<UserId>(path_info.into_inner().0)?;


    let requested_user_exists =
        entities::UserQuery::exists_by_id(&mut database_connection, requested_user_id).await?;

    if !requested_user_exists {
        return EndpointResponseBuilder::not_found()
            .with_error_reason(UsersErrorReason::user_not_found())
            .build();
    }


    let requested_user_permission_set = entities::UserRoleQuery::transitive_permissions_for_user(
        &mut database_connection,
        requested_user_id,
    )
    .await?;


    EndpointResponseBuilder::ok()
        .with_json_body(UserPermissionsResponse {
            permissions: requested_user_permission_set.permission_names_cow(),
        })
        .build()
}




declare_openapi_error_reason_response!(
    pub struct SpecificUserCannotModifyYourself {
        description => "You are not allowed to change your own account on this endpoint.",
        reason => UsersErrorReason::cannot_modify_your_own_account()
    }
);

declare_openapi_error_reason_response!(
    pub struct SpecificUserNewDisplayNameAlreadyExists {
        description => "Unable to change user's display name, because the \
                        given display name is already in use.",
        reason => UsersErrorReason::display_name_already_exists()
    }
);



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
            "user_id" = String,
            Path,
            format = Uuid,
            description = "UUID of the user to change the display name for."
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
                    "id": "01921e73-08f4-7d7c-9e24-769ebe361edc",
                    "username": "janeznovak",
                    "display_name": "Janez Novak Veliki",
                    "joined_at": "2023-06-27T20:33:53.078789Z",
                    "last_modified_at": "2023-06-27T20:44:27.217273Z",
                    "last_active_at": "2023-06-27T20:34:27.253746Z"
                }
            })
        ),
        (
            status = 403,
            response = inline(AsErrorReason<SpecificUserCannotModifyYourself>)
        ),
        (
            status = 404,
            response = inline(AsErrorReason<SpecificUserNotFound>)
        ),
        (
            status = 409,
            response = inline(AsErrorReason<SpecificUserNewDisplayNameAlreadyExists>)
        ),
        openapi::response::UuidUrlParameterError,
        openapi::response::RequiredJsonBodyErrors,
        openapi::response::MissingAuthentication,
        openapi::response::MissingPermissions<requires::UserAnyWrite, 1>,
        openapi::response::InternalServerError,
    ),
    security(
        ("access_token" = [])
    )
)]
#[patch("/{user_id}/display_name")]
async fn update_specific_user_display_name(
    state: ApplicationState,
    authentication_extractor: UserAuthenticationExtractor,
    path_info: web::Path<(String,)>,
    request_data: web::Json<UserDisplayNameChangeRequest>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;
    let mut transaction = database_connection.transaction().begin().await?;


    // To access this endpoint, the user:
    // - MUST provide an authentication token, and
    // - MUST have the `user.self:write` permission.
    //
    // Intended for moderation tooling.
    let authenticated_user = require_user_authentication_and_permissions!(
        &mut transaction,
        authentication_extractor,
        Permission::UserAnyWrite
    );

    let authenticated_user_id = authenticated_user.user_id();

    let requested_user_id = parse_uuid::<UserId>(path_info.into_inner().0)?;
    let request_data = request_data.into_inner();


    // Disallow modifying your own account on these `/{user_id}/*` endpoints.
    if authenticated_user_id == requested_user_id {
        return EndpointResponseBuilder::forbidden()
            .with_error_reason(UsersErrorReason::cannot_modify_your_own_account())
            .build();
    }


    let requested_user_exists =
        entities::UserQuery::exists_by_id(&mut transaction, requested_user_id).await?;

    if !requested_user_exists {
        return EndpointResponseBuilder::not_found()
            .with_error_reason(UsersErrorReason::user_not_found())
            .build();
    }



    // Modify requested user's display name.
    let new_display_name_already_exists = entities::UserQuery::exists_by_display_name(
        &mut transaction,
        &request_data.new_display_name,
    )
    .await?;

    if new_display_name_already_exists {
        return EndpointResponseBuilder::conflict()
            .with_error_reason(UsersErrorReason::display_name_already_exists())
            .build();
    }


    // Update requested user's display name.
    let updated_user = entities::UserMutation::change_display_name_by_user_id(
        &mut transaction,
        requested_user_id,
        &request_data.new_display_name,
    )
    .await?;


    transaction.commit().await?;

    info!(
        operator_id = %authenticated_user_id,
        target_user_id = %requested_user_id,
        new_display_name = %request_data.new_display_name,
        "User has updated another user's display name."
    );


    EndpointResponseBuilder::ok()
        .with_json_body(UserDisplayNameChangeResponse {
            user: updated_user.into_api_model(),
        })
        .build()
}



declare_openapi_error_reason_response!(
    pub struct SpecificUserInvalidRoleName {
        description => "Invalid role name.",
        reason => UsersErrorReason::invalid_role_name(String::default())
    }
);

declare_openapi_error_reason_response!(
    pub struct SpecificUserCannotGiveOutRolesYouDontHave {
        description => "Not allowed to add a given role, because you do not have it.",
        reason => UsersErrorReason::unable_to_give_out_unowned_role(Role::User)
    }
);


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
            "user_id" = String,
            Path,
            format = Uuid,
            description = "UUID of the user to add roles to."
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
            response = inline(AsErrorReason<SpecificUserInvalidRoleName>)
        ),
        (
            status = 403,
            response = inline(AsErrorReason<SpecificUserCannotGiveOutRolesYouDontHave>)
        ),
        (
            status = 403,
            response = inline(AsErrorReason<SpecificUserCannotModifyYourself>)
        ),
        (
            status = 404,
            response = inline(AsErrorReason<SpecificUserNotFound>)
        ),
        openapi::response::UuidUrlParameterError,
        openapi::response::RequiredJsonBodyErrors,
        openapi::response::MissingAuthentication,
        openapi::response::MissingPermissions<requires::UserAnyWrite, 1>,
        openapi::response::InternalServerError,
    ),
    security(
        ("access_token" = [])
    )
)]
#[post("/{user_id}/roles")]
pub async fn add_roles_to_specific_user(
    state: ApplicationState,
    authentication_extractor: UserAuthenticationExtractor,
    path_info: web::Path<(String,)>,
    json_data: web::Json<UserRoleAddRequest>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;
    let mut transaction = database_connection.transaction().begin().await?;


    // To access this endpoint, the user:
    // - MUST provide an authentication token, and
    // - MUST have the `user.self:write` permission.
    //
    // Intended for moderation tooling.
    let authenticated_user = require_user_authentication!(authentication_extractor);
    let authenticated_user_roles = authenticated_user.fetch_roles(&mut transaction).await?;
    let authenticated_user_effective_permissions = authenticated_user_roles.granted_permission_set();

    require_permission_in_set!(
        authenticated_user_effective_permissions,
        Permission::UserAnyWrite
    );


    let authenticated_user_id = authenticated_user.user_id();

    let requested_user_id = parse_uuid::<UserId>(path_info.into_inner().0)?;
    let request_data = json_data.into_inner();


    // Disallow modifying your own user account on this endpoint.
    if authenticated_user_id == requested_user_id {
        return EndpointResponseBuilder::forbidden()
            .with_error_reason(UsersErrorReason::cannot_modify_your_own_account())
            .build();
    }


    let roles_to_add_to_user = {
        let mut roles_to_add_to_user = HashSet::with_capacity(request_data.roles_to_add.len());

        for raw_role_name in request_data.roles_to_add {
            let Some(role) = Role::from_name(&raw_role_name) else {
                return EndpointResponseBuilder::bad_request()
                    .with_error_reason(UsersErrorReason::invalid_role_name(raw_role_name))
                    .build();
            };

            roles_to_add_to_user.insert(role);
        }

        RoleSet::from_role_hash_set(roles_to_add_to_user)
    };


    // Validate that the authenticated user has all of the roles
    // they wish to assign to other users. Not checking for this would
    // be dangerous as it would essentially allow for privilege escalation.
    for role in roles_to_add_to_user.roles() {
        if !authenticated_user_roles.has_role(role) {
            return EndpointResponseBuilder::forbidden()
                .with_error_reason(UsersErrorReason::unable_to_give_out_unowned_role(
                    *role,
                ))
                .build();
        }
    }



    let user_exists = entities::UserQuery::exists_by_id(&mut transaction, requested_user_id).await?;

    if !user_exists {
        return EndpointResponseBuilder::not_found()
            .with_error_reason(UsersErrorReason::user_not_found())
            .build();
    }


    let full_updated_user_role_set = entities::UserRoleMutation::add_roles_to_user(
        &mut transaction,
        requested_user_id,
        roles_to_add_to_user.clone(),
    )
    .await?;


    transaction.commit().await?;

    info!(
        operator_id = %authenticated_user_id,
        updated_user_id = %requested_user_id,
        roles_added = ?roles_to_add_to_user,
        new_updated_role_set = ?full_updated_user_role_set,
        "Added roles to user."
    );


    EndpointResponseBuilder::ok()
        .with_json_body(UserRolesResponse {
            role_names: full_updated_user_role_set.role_names_cow(),
        })
        .build()
}



declare_openapi_error_reason_response!(
    pub struct SpecificUserCannotTakeAwayRolesYouDontHave {
        description => "Not allowed to remove a given role, because you do not have it.",
        reason => UsersErrorReason::unable_to_take_away_unowned_role(Role::User)
    }
);


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
            format = Uuid,
            description = "UUID of the user to remove roles from."
        )
    ),
    params(
        UserRoleRemoveRequest
    ),
    responses(
        (
            status = 200,
            description = "Updated user role list.",
            body = UserRolesResponse
        ),
        (
            status = 400,
            response = inline(AsErrorReason<SpecificUserInvalidRoleName>)
        ),
        (
            status = 403,
            response = inline(AsErrorReason<SpecificUserCannotTakeAwayRolesYouDontHave>)
        ),
        (
            status = 403,
            response = inline(AsErrorReason<SpecificUserCannotModifyYourself>)
        ),
        (
            status = 404,
            response = inline(AsErrorReason<SpecificUserNotFound>)
        ),
        openapi::response::UuidUrlParameterError,
        openapi::response::RequiredJsonBodyErrors,
        openapi::response::MissingAuthentication,
        openapi::response::MissingPermissions<requires::UserAnyWrite, 1>,
        openapi::response::InternalServerError,
    ),
    security(
        ("access_token" = [])
    )
)]
#[delete("/{user_id}/roles")]
pub async fn remove_roles_from_specific_user(
    state: ApplicationState,
    authentication_extractor: UserAuthenticationExtractor,
    path_info: web::Path<(String,)>,
    request_data: web::Query<UserRoleRemoveRequest>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;
    let mut transaction = database_connection.transaction().begin().await?;


    // To access this endpoint, the user:
    // - MUST provide an authentication token, and
    // - MUST have the `user.self:write` permission.
    //
    // Intended for moderation tooling.
    let authenticated_user = require_user_authentication!(authentication_extractor);
    let authenticated_user_roles = authenticated_user.fetch_roles(&mut transaction).await?;
    let authenticated_user_effective_permissions = authenticated_user_roles.granted_permission_set();

    require_permission_in_set!(
        authenticated_user_effective_permissions,
        Permission::UserAnyWrite
    );


    let authenticated_user_id = authenticated_user.user_id();

    let requested_user_id = parse_uuid::<UserId>(path_info.into_inner().0)?;
    let request_data = request_data.into_inner();


    // Disallow modifying your own user account on this endpoint.
    if authenticated_user_id == requested_user_id {
        return EndpointResponseBuilder::forbidden()
            .with_error_reason(UsersErrorReason::cannot_modify_your_own_account())
            .build();
    }


    let roles_to_remove_from_user = {
        let mut roles_to_remove_from_user =
            HashSet::with_capacity(request_data.roles_to_remove.len());

        for raw_role_name in request_data.roles_to_remove {
            let Some(role) = Role::from_name(&raw_role_name) else {
                return EndpointResponseBuilder::bad_request()
                    .with_error_reason(UsersErrorReason::invalid_role_name(raw_role_name))
                    .build();
            };

            roles_to_remove_from_user.insert(role);
        }

        RoleSet::from_role_hash_set(roles_to_remove_from_user)
    };


    // Validate that the authenticated user (caller) has all of the roles
    // they wish to remove from the target user. Not checking for this would
    // be dangerous as it would essentially allow for privilege de-escalation.
    for role in roles_to_remove_from_user.roles() {
        if !authenticated_user_roles.has_role(role) {
            return EndpointResponseBuilder::forbidden()
                .with_error_reason(UsersErrorReason::unable_to_take_away_unowned_role(*role))
                .build();
        }
    }


    let user_exists = entities::UserQuery::exists_by_id(&mut transaction, requested_user_id).await?;

    if !user_exists {
        return EndpointResponseBuilder::not_found()
            .with_error_reason(UsersErrorReason::user_not_found())
            .build();
    }


    let full_updated_user_role_set = entities::UserRoleMutation::remove_roles_from_user(
        &mut transaction,
        requested_user_id,
        roles_to_remove_from_user.clone(),
    )
    .await?;


    transaction.commit().await?;

    info!(
        operator_id = %authenticated_user_id,
        updated_user_id = %requested_user_id,
        roles_removed = ?roles_to_remove_from_user,
        new_updated_role_set = ?full_updated_user_role_set,
        "Removed roles from user."
    );


    EndpointResponseBuilder::ok()
        .with_json_body(UserRolesResponse {
            role_names: full_updated_user_role_set.role_names_cow(),
        })
        .build()
}
