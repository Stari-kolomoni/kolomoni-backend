use actix_web::{get, patch, web};
use kolomoni_core::api_models::{
    UserDisplayNameChangeRequest,
    UserDisplayNameChangeResponse,
    UserInfoResponse,
    UserPermissionsResponse,
    UserRolesResponse,
};
use kolomoni_core::permissions::Permission;
use kolomoni_database::entities;
use sqlx::Acquire;
use tracing::info;

use crate::{
    api::{
        errors::{EndpointResponseBuilder, EndpointResult, UsersErrorReason},
        openapi::{
            self,
            response::{requires, AsErrorReason},
        },
        traits::IntoApiModel,
        OptionalIfModifiedSince,
    },
    authentication::UserAuthenticationExtractor,
    declare_openapi_error_reason_response,
    require_permission_in_set,
    require_user_authentication,
    require_user_authentication_and_permissions,
    state::ApplicationState,
};



declare_openapi_error_reason_response!(
    pub struct UserYourAccountNotFound {
        description => "Your user account no longer exists.",
        reason => UsersErrorReason::user_not_found()
    }
);


/// Get your user information
///
/// This endpoint returns the logged-in user's information.
///
///
/// # Authentication
/// This endpoint requires authentication and the `users.self:read` permission.
#[utoipa::path(
    get,
    path = "/users/me",
    tag = "users:self",
    params(
        openapi::param::IfModifiedSince
    ),
    responses(
        (
            status = 200,
            description
                = "Information about the current user \
                  (i.e. the user who owns the authentication token used in the request).",
            body = UserInfoResponse,
            headers(
                (
                    "Last-Modified" = String,
                    description = "Last user modification time. You may use this value for caching purposes."
                )
            )
        ),
        (
            status = 404,
            response = inline(AsErrorReason<UserYourAccountNotFound>)
        ),
        openapi::response::Unmodified,
        openapi::response::MissingAuthentication,
        openapi::response::MissingPermissions<requires::UserSelfRead, 1>,
        openapi::response::InternalServerError,
    ),
    security(
        ("access_token" = [])
    )
)]
#[get("/me")]
pub async fn get_current_user_info(
    state: ApplicationState,
    authentication_extractor: UserAuthenticationExtractor,
    if_modified_since_header: OptionalIfModifiedSince,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;


    // To access this endpoint, the user:
    // - MUST provide an authentication token, and
    // - MUST have the `user.self:read` permission.
    let authenticated_user = require_user_authentication_and_permissions!(
        &mut database_connection,
        authentication_extractor,
        Permission::UserSelfRead
    );

    let authenticated_user_id = authenticated_user.user_id();

    // Load user from database.
    let Some(current_user) =
        entities::UserQuery::get_user_by_id(&mut database_connection, authenticated_user_id).await?
    else {
        return EndpointResponseBuilder::not_found()
            .with_error_reason(UsersErrorReason::user_not_found())
            .build();
    };


    let user_last_modified_at = current_user.last_modified_at;

    if if_modified_since_header.enabled_and_has_not_changed_since(&user_last_modified_at) {
        return EndpointResponseBuilder::not_modified()
            .with_last_modified_at(&user_last_modified_at)
            .build();
    }


    EndpointResponseBuilder::ok()
        .with_json_body(UserInfoResponse {
            user: current_user.into_api_model(),
        })
        .with_last_modified_at(&user_last_modified_at)
        .build()
}




/// Get your roles
///
/// This endpoint returns the logged-in user's role list.
///
///
/// # Authentication
/// This endpoint requires authentication and the `users.self:read` permission.
#[utoipa::path(
    get,
    path = "/users/me/roles",
    tag = "users:self",
    responses(
        (
            status = 200,
            description = "List of roles for the authenticated user.",
            body = UserRolesResponse
        ),
        (
            status = 404,
            response = inline(AsErrorReason<UserYourAccountNotFound>)
        ),
        openapi::response::MissingAuthentication,
        openapi::response::MissingPermissions<requires::UserSelfRead, 1>,
        openapi::response::InternalServerError,
    ),
    security(
        ("access_token" = [])
    )
)]
#[get("/me/roles")]
pub async fn get_current_user_roles(
    state: ApplicationState,
    authentication_extractor: UserAuthenticationExtractor,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;


    // To access this endpoint, the user:
    // - MUST provide an authentication token, and
    // - MUST have the `user.self:read` permission.
    let authenticated_user = require_user_authentication_and_permissions!(
        &mut database_connection,
        authentication_extractor,
        Permission::UserSelfRead
    );

    let authenticated_user_id = authenticated_user.user_id();


    let user_exists =
        entities::UserQuery::exists_by_id(&mut database_connection, authenticated_user_id).await?;

    if !user_exists {
        return EndpointResponseBuilder::not_found()
            .with_error_reason(UsersErrorReason::user_not_found())
            .build();
    }


    let user_roles =
        entities::UserRoleQuery::roles_for_user(&mut database_connection, authenticated_user_id)
            .await?;


    EndpointResponseBuilder::ok()
        .with_json_body(UserRolesResponse {
            role_names: user_roles.role_names_cow(),
        })
        .build()
}



/// Get your effective permissions
///
/// This endpoint returns the logged-in user's effective permission list.
/// The effective permission list depends on permissions that each of the user's roles provide.
///
/// # Authentication
/// This endpoint requires authentication and the `users.self:read` permission.
#[utoipa::path(
    get,
    path = "/users/me/permissions",
    tag = "users:self",
    responses(
        (
            status = 200,
            description = "A list of your permissions.",
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
            status = 404,
            response = inline(AsErrorReason<UserYourAccountNotFound>)
        ),
        openapi::response::MissingAuthentication,
        openapi::response::MissingPermissions<requires::UserSelfRead, 1>,
        openapi::response::InternalServerError,
    ),
    security(
        ("access_token" = [])
    )
)]
#[get("/me/permissions")]
async fn get_current_user_effective_permissions(
    state: ApplicationState,
    authentication_extractor: UserAuthenticationExtractor,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;

    // To access this endpoint, the user:
    // - MUST provide an authentication token, and
    // - MUST have the `user.self:read` permission.
    let authenticated_user = require_user_authentication!(authentication_extractor);
    let user_permissions = authenticated_user
        .fetch_transitive_permissions(&mut database_connection)
        .await?;

    require_permission_in_set!(user_permissions, Permission::UserSelfRead);


    let user_exists = entities::UserQuery::exists_by_id(
        &mut database_connection,
        authenticated_user.user_id(),
    )
    .await?;

    if !user_exists {
        return EndpointResponseBuilder::not_found()
            .with_error_reason(UsersErrorReason::user_not_found())
            .build();
    }


    EndpointResponseBuilder::ok()
        .with_json_body(UserPermissionsResponse {
            permissions: user_permissions.permission_names_cow(),
        })
        .build()
}




declare_openapi_error_reason_response!(
    pub struct UserNewDisplayNameAlreadyExists {
        description => "Unable to change display name, because the \
                        given display name is already in use.",
        reason => UsersErrorReason::display_name_already_exists()
    }
);


/// Change your display name
///
/// This endpoint allows you to change your own display name. Note that the display name
/// must be unique among all users, so your request may be denied with a `409 Conflict`
/// to indicate a display name collision.
///
/// # Authentication
/// This endpoint requires the `users.self:write` permission.
#[utoipa::path(
    patch,
    path = "/users/me/display_name",
    tag = "users:self",
    request_body(
        content = UserDisplayNameChangeRequest,
    ),
    responses(
        (
            status = 200,
            description = "Your display name has been changed.",
            body = UserDisplayNameChangeResponse,
        ),
        (
            status = 409,
            response = inline(AsErrorReason<UserNewDisplayNameAlreadyExists>)
        ),
        (
            status = 404,
            response = inline(AsErrorReason<UserYourAccountNotFound>)
        ),
        openapi::response::RequiredJsonBodyErrors,
        openapi::response::MissingAuthentication,
        openapi::response::MissingPermissions<requires::UserSelfRead, 1>,
        openapi::response::InternalServerError,
    ),
    security(
        ("access_token" = [])
    )
)]
#[patch("/me/display_name")]
async fn update_current_user_display_name(
    state: ApplicationState,
    authentication_extractor: UserAuthenticationExtractor,
    request_data: web::Json<UserDisplayNameChangeRequest>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;
    let mut transaction = database_connection.begin().await?;


    // To access this endpoint, the user:
    // - MUST provide an authentication token, and
    // - MUST have the `user.self:write` permission.
    let authenticated_user = require_user_authentication_and_permissions!(
        &mut transaction,
        authentication_extractor,
        Permission::UserSelfWrite
    );

    let authenticated_user_id = authenticated_user.user_id();
    let request_data = request_data.into_inner();



    // Ensure the display name is unique.
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


    // Update user in the database.
    let updated_user = entities::UserMutation::change_display_name_by_user_id(
        &mut transaction,
        authenticated_user_id,
        &request_data.new_display_name,
    )
    .await?;


    transaction.commit().await?;


    info!(
        user_id = %authenticated_user_id,
        new_display_name = request_data.new_display_name,
        "User has updated their display name."
    );


    EndpointResponseBuilder::ok()
        .with_json_body(UserDisplayNameChangeResponse {
            user: updated_user.into_api_model(),
        })
        .build()
}
