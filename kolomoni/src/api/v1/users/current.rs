use actix_web::{get, http::StatusCode, patch, web};
use kolomoni_auth::Permission;
use kolomoni_core::api_models::UserDisplayNameChangeRequest;
use kolomoni_database::entities;
use sqlx::Acquire;
use tracing::info;

use crate::{
    api::{
        errors::{APIError, EndpointResult},
        macros::{
            construct_not_modified_response,
            ContextlessResponder,
            IntoKolomoniResponseBuilder,
        },
        openapi,
        traits::IntoApiModel,
        v1::users::{
            UserDisplayNameChangeResponse,
            UserInfoResponse,
            UserPermissionsResponse,
            UserRolesResponse,
        },
        OptionalIfModifiedSince,
    },
    authentication::UserAuthenticationExtractor,
    json_error_response_with_reason,
    obtain_database_connection,
    require_authentication,
    require_permission,
    state::ApplicationState,
};

// TODO introduce transactions here and elsewhere (even in read-only operations, for consistency)


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
        openapi::IfModifiedSinceParameter
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
                    description = "Last user modification time. Use this value for caching."
                )
            )
        ),
        (
            status = 404,
            description = "The user no longer exists."
        ),
        openapi::InternalServerErrorResponse,
        openapi::UnmodifiedConditionalResponse,
        openapi::FailedAuthenticationResponses<openapi::RequiresUserSelfRead>,
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
    let mut database_connection = obtain_database_connection!(state);


    // To access this endpoint, the user:
    // - MUST provide an authentication token, and
    // - MUST have the `user.self:read` permission.
    let authenticated_user = require_authentication!(authentication_extractor);
    require_permission!(
        &mut database_connection,
        authenticated_user,
        Permission::UserSelfRead
    );


    let authenticated_user_id = authenticated_user.user_id();


    // Load user from database.
    let current_user =
        entities::UserQuery::get_user_by_id(&mut database_connection, authenticated_user_id)
            .await?
            .ok_or_else(APIError::not_found)?;


    if if_modified_since_header.enabled_and_has_not_changed_since(&current_user.last_modified_at) {
        construct_not_modified_response(&current_user.last_modified_at)
    } else {
        let last_modified_at = current_user.last_modified_at;

        Ok(UserInfoResponse {
            user: current_user.into_api_model(),
        }
        .into_response_builder()?
        .last_modified_at(last_modified_at)?
        .build())
    }
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
            description = "The authenticated user's role list.",
            body = UserRolesResponse
        ),
        (
            status = 404,
            description = "You do not exist."
        ),
        openapi::FailedAuthenticationResponses<openapi::RequiresUserAnyRead>,
        openapi::InternalServerErrorResponse,
    ),
    security(
        ("access_token" = [])
    )
)]
#[get("/me/roles")]
pub async fn get_current_user_roles(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
) -> EndpointResult {
    let mut database_connection = obtain_database_connection!(state);


    let authenticated_user = require_authentication!(authentication);
    require_permission!(
        &mut database_connection,
        authenticated_user,
        Permission::UserSelfRead
    );


    let user_exists = entities::UserQuery::exists_by_id(
        &mut database_connection,
        authenticated_user.user_id(),
    )
    .await?;

    if !user_exists {
        return Err(APIError::not_found());
    }


    let user_roles = entities::UserRoleQuery::roles_for_user(
        &mut database_connection,
        authenticated_user.user_id(),
    )
    .await?;

    let user_role_names = user_roles.role_names();


    Ok(UserRolesResponse {
        role_names: user_role_names,
    }
    .into_response())
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
        openapi::FailedAuthenticationResponses<openapi::RequiresUserSelfRead>,
        openapi::InternalServerErrorResponse,
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
    let mut database_connection = obtain_database_connection!(state);


    // User must be authenticated and
    // have the `user.self:read` permission to access this endpoint.
    let authenticated_user = require_authentication!(authentication_extractor);
    let user_permissions = authenticated_user
        .fetch_transitive_permissions(&mut database_connection)
        .await?;

    require_permission!(user_permissions, Permission::UserSelfRead);


    let permission_names = user_permissions
        .permission_names()
        .into_iter()
        .map(|name_static_str| name_static_str.to_string())
        .collect();

    Ok(UserPermissionsResponse::from_permission_names(permission_names).into_response())
}



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
        example = json!({
            "new_display_name": "Janez Novak Veliki"
        })
    ),
    responses(
        (
            status = 200,
            description = "Your display name has been changed.",
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
            status = 409,
            description = "User with given display name already exists.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "User with given display name already exists." })
        ),
        openapi::MissingOrInvalidJsonRequestBodyResponse,
        openapi::FailedAuthenticationResponses<openapi::RequiresUserSelfWrite>,
        openapi::InternalServerErrorResponse,
    ),
    security(
        ("access_token" = [])
    )
)]
#[patch("/me/display_name")]
async fn update_current_user_display_name(
    state: ApplicationState,
    authentication_extractor: UserAuthenticationExtractor,
    json_data: web::Json<UserDisplayNameChangeRequest>,
) -> EndpointResult {
    let mut database_connection = obtain_database_connection!(state);
    let mut transaction = database_connection.begin().await?;


    // User must be authenticated and have
    // the `user.self:write` permission to access this endpoint.
    let authenticated_user = require_authentication!(authentication_extractor);
    require_permission!(
        &mut transaction,
        authenticated_user,
        Permission::UserSelfWrite
    );


    let authenticated_user_id = authenticated_user.user_id();
    let json_data = json_data.into_inner();



    // Ensure the display name is unique.
    let display_name_already_exists =
        entities::UserQuery::exists_by_display_name(&mut *transaction, &json_data.new_display_name)
            .await?;

    if display_name_already_exists {
        return Ok(json_error_response_with_reason!(
            StatusCode::CONFLICT,
            "User with given display name already exists."
        ));
    }


    // Update user in the database.
    let updated_user = entities::UserMutation::change_display_name_by_user_id(
        &mut *transaction,
        authenticated_user_id,
        &json_data.new_display_name,
    )
    .await?;


    transaction.commit().await?;


    info!(
        user_id = %authenticated_user_id,
        new_display_name = json_data.new_display_name,
        "User has updated their display name."
    );


    Ok(UserDisplayNameChangeResponse {
        user: updated_user.into_api_model(),
    }
    .into_response())
}
