use actix_web::{get, http::StatusCode, patch, web};
use kolomoni_auth::permissions::Permission;
use kolomoni_database::{mutation, query};
use sea_orm::TransactionTrait;
use tracing::info;

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


/// Get current user's information
///
/// This endpoint returns the logged-in user's information.
///
///
/// # Required permissions
/// This endpoint requires the `users.self:read` permission.
#[utoipa::path(
    get,
    path = "/users/me",
    tag = "self",
    responses(
        (
            status = 200,
            description = "Information about current user.",
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
            description = "Missing `user.self:read` permission.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "Missing permission: user.self:read." })
        ),
        (
            status = 404,
            description = "The user no longer exists."
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
#[get("/me")]
pub async fn get_current_user_info(state: ApplicationState, user_auth: UserAuth) -> EndpointResult {
    // User must provide an authentication token and
    // have the `user.self:read` permission to access this endpoint.
    let (token, permissions) = require_authentication!(state, user_auth);
    require_permission!(permissions, Permission::UserSelfRead);


    // Load user from database.
    let user = query::UserQuery::get_user_by_username(&state.database, &token.username)
        .await
        .map_err(APIError::InternalError)?
        .ok_or_else(APIError::not_found)?;


    Ok(UserInfoResponse::new(user).into_response())
}




/// Get current user's permissions
///
/// # Required permissions
/// This endpoint requires the `users.self:read` permission.
#[utoipa::path(
    get,
    path = "/users/me/permissions",
    tag = "self",
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
            status = 401,
            description = "Missing user authentication."
        ),
        (
            status = 403,
            description = "Missing `user.self:read` permission.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "Missing permission: user.self:read." })
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
#[get("/me/permissions")]
async fn get_current_user_permissions(
    state: ApplicationState,
    user_auth: UserAuth,
) -> EndpointResult {
    // User must be authenticated and
    // have the `user.self:read` permission to access this endpoint.
    let (_, permissions) = require_authentication!(state, user_auth);
    require_permission!(permissions, Permission::UserSelfRead);


    Ok(UserPermissionsResponse {
        permissions: permissions.to_permission_names(),
    }
    .into_response())
}



/// Change the current user's display name
///
/// This endpoint allows you to change your own display name. Note that the display name
/// must be unique among all users, so your request may be denied with a `409 Conflict`
/// to indicate a display name collision.
///
/// # Required permissions
/// This endpoint requires the `users.self:write` permission.
#[utoipa::path(
    patch,
    path = "/users/me/display_name",
    tag = "self",
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
            status = 401,
            description = "Missing user authentication."
        ),
        (
            status = 403,
            description = "Missing `user.self:write` permission.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "Missing permission: user.self:write." })
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
#[patch("/me/display_name")]
async fn update_current_user_display_name(
    state: ApplicationState,
    user_auth: UserAuth,
    json_data: web::Json<UserDisplayNameChangeRequest>,
) -> EndpointResult {
    // User must be authenticated and have
    // the `user.self:write` permission to access this endpoint.
    let (token, permissions) = require_authentication!(state, user_auth);
    require_permission!(permissions, Permission::UserSelfWrite);


    let json_data = json_data.into_inner();
    let database_transaction = state
        .database
        .begin()
        .await
        .map_err(APIError::InternalDatabaseError)?;


    // Ensure the display name is unique.
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


    // Update user in the database.
    mutation::UserMutation::update_display_name_by_username(
        &database_transaction,
        &token.username,
        json_data.new_display_name.clone(),
    )
    .await
    .map_err(APIError::InternalError)?;

    // TODO Consider merging this update into all mutation methods where it makes sense.
    //      Otherwise we're wasting a round-trip to the database for no real reason.
    let updated_user = mutation::UserMutation::update_last_active_at_by_username(
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
        user: UserInformation::from_user_model(updated_user),
    }
    .into_response())
}
