use actix_web::{
    get,
    http::{header, StatusCode},
    patch,
    web,
    HttpResponse,
};
use kolomoni_auth::Permission;
use kolomoni_database::{mutation, query};
use miette::IntoDiagnostic;
use sea_orm::TransactionTrait;
use tracing::info;

use crate::{
    api::{
        errors::{APIError, EndpointResult},
        macros::{
            construct_last_modified_header_value,
            ContextlessResponder,
            IntoKolomoniResponseBuilder,
        },
        v1::users::{
            UserDisplayNameChangeRequest,
            UserDisplayNameChangeResponse,
            UserInfoResponse,
            UserInformation,
            UserPermissionsResponse,
        },
        OptionalIfModifiedSince,
    },
    authentication::UserAuth,
    error_response_with_reason,
    require_authentication,
    require_permission,
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
    params(
        (
            "If-Modified-Since" = Option<String>,
            Header,
            description = "If specified, this header makes the server return `304 Not Modified` if \
                           the user's data hasn't changed since the specified timestamp. \
                           See [this article on MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/If-Modified-Since) \
                           for more information about this conditional header.",
            example = "Wed, 21 Oct 2015 07:28:00 GMT"
        )
    ),
    responses(
        (
            status = 200,
            description = "Information about the current user (i.e. the user who owns the authentication token used in the request).",
            body = UserInfoResponse,
            headers(
                ("Last-Modified" = String, description = "Last user modification time. Use this value for caching.")
            ),
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
            status = 304,
            description = "User hasn't been modified since the timestamp specified in the `If-Modified-Since` header. \
                           As such, this status code can only be returned if that header is provided in the request."
        ),
        (
            status = 401,
            description = "Missing authentication. Include an `Authorization: Bearer <token>` header \
                           with your request to access this endpoint."
        ),
        (
            status = 403,
            description = "Missing the `user.self:read` permission.",
            content_type = "application/json",
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
pub async fn get_current_user_info(
    state: ApplicationState,
    user_auth: UserAuth,
    modified_since_conditional: OptionalIfModifiedSince,
) -> EndpointResult {
    // User must provide an authentication token and
    // have the `user.self:read` permission to access this endpoint.
    let (token, permissions) = require_authentication!(state, user_auth);
    require_permission!(permissions, Permission::UserSelfRead);


    // Load user from database.
    let user = query::UserQuery::get_user_by_username(&state.database, &token.username)
        .await
        .map_err(APIError::InternalError)?
        .ok_or_else(APIError::not_found)?;

    let last_modification_time = user.last_modified_at.to_utc();

    if modified_since_conditional.is_unchanged(&last_modification_time) {
        let mut unchanged_response = HttpResponse::new(StatusCode::NOT_MODIFIED);

        unchanged_response.headers_mut().append(
            header::LAST_MODIFIED,
            construct_last_modified_header_value(&last_modification_time)
                .into_diagnostic()
                .map_err(APIError::InternalError)?,
        );

        Ok(unchanged_response)
    } else {
        Ok(UserInfoResponse::new(user)
            .into_response_builder()?
            .last_modified_at(last_modification_time)?
            .build())
    }
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


    Ok(
        UserPermissionsResponse::from_permission_names(permissions.to_permission_names())
            .into_response(),
    )
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
        return Ok(error_response_with_reason!(
            StatusCode::CONFLICT,
            "User with given display name already exists."
        ));
    }


    // Update user in the database.
    let updated_user = mutation::UserMutation::update_display_name_by_username(
        &database_transaction,
        &token.username,
        json_data.new_display_name.clone(),
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
