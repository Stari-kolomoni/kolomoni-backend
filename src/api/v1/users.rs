use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{delete, HttpResponseBuilder};
use actix_web::{get, patch, post, web, HttpRequest, HttpResponse, Responder, Scope};
use anyhow::Result;
use chrono::{DateTime, Utc};
use sea_orm::TransactionTrait;
use serde::{Deserialize, Serialize};
use tracing::info;
use utoipa::ToSchema;

use crate::api::auth::{UserAuth, UserPermission, UserPermissions};
use crate::api::errors::{APIError, EndpointResult, ErrorReasonResponse};
use crate::api::macros::DumbResponder;
use crate::database::mutation::UserRegistrationInfo;
use crate::database::{entities, mutation, query};
use crate::state::AppState;
use crate::{impl_json_responder, require_permission, response_with_reason};

/*
 * Shared
 */

/// Information about a single user.
#[derive(Serialize, Debug, ToSchema)]
pub struct UserInformation {
    /// Internal user ID.
    pub id: i32,

    /// Unique username for login.
    pub username: String,

    /// Unique display name.
    pub display_name: String,

    /// Registration date and time.
    pub joined_at: DateTime<Utc>,

    /// Last modification date and time.
    pub last_modified_at: DateTime<Utc>,

    /// Last activity date and time.
    pub last_active_at: DateTime<Utc>,
}

impl UserInformation {
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


/// Information about a single user.
#[derive(Serialize, Debug, ToSchema)]
pub struct UserInfoResponse {
    pub user: UserInformation,
}

impl UserInfoResponse {
    pub fn new(model: entities::users::Model) -> Self {
        Self {
            user: UserInformation::from_user_model(model),
        }
    }
}

impl_json_responder!(UserInfoResponse);



/// Request to change a user's display name.
#[derive(Deserialize, Clone, Debug, ToSchema)]
pub struct UserDisplayNameChangeRequest {
    pub new_display_name: String,
}


/// Response indicating successful change of a display name.
/// Contains the updated user information.
#[derive(Serialize, Debug, ToSchema)]
pub struct UserDisplayNameChangeResponse {
    pub user: UserInformation,
}

impl_json_responder!(UserDisplayNameChangeResponse);


/// Response containing a list of active permissions.
#[derive(Serialize, Debug, ToSchema)]
pub struct UserPermissionsResponse {
    pub permissions: Vec<String>,
}

impl_json_responder!(UserPermissionsResponse);



/*
 * GET /users
 */

/// List of registered users.
#[derive(Serialize, Debug, ToSchema)]
pub struct RegisteredUsersListResponse {
    pub users: Vec<UserInformation>,
}

impl_json_responder!(RegisteredUsersListResponse);

// # Development note
//
// We use "" instead of "/" below (in `#[get("")`) and in other places
// because this allows the user to request `GET /api/v1/users` OR `GET /api/v1/users/` and
// get the correct endpoint both times.
//
// For more information, see `actix_web::middleware::NormalizePath` (trim mode).

/// List users
///
/// This endpoint returns a list of all registered users.
///
/// *This endpoint requires the `users.any:read` permission.*
#[utoipa::path(
    get,
    path = "/users",
    tag = "users",
    responses(
        (
            status = 200,
            description = "List of registered users.",
            body = inline(RegisteredUsersListResponse),
            example = json!({
                "users": [
                    {
                        "id": 1,
                        "username": "janeznovak",
                        "display_name": "Janez Novak",
                        "joined_at": "2023-06-27T20:33:53.078789Z",
                        "last_modified_at": "2023-06-27T20:34:27.217273Z",
                        "last_active_at": "2023-06-27T20:34:27.253746Z"
                    },
                ]
            }),
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
            status = 500,
            description = "Internal server error."
        )
    ),
    security(
        ("access_token" = [])
    )
)]
#[get("")]
async fn get_all_registered_users(
    user_auth: UserAuth,
    state: web::Data<AppState>,
) -> EndpointResult {
    // User must provide the authentication token and
    // have the `user.any:read` permission to access this endpoint.
    let (_, permissions) = user_auth
        .token_and_permissions_if_authenticated(&state.database)
        .await
        .map_err(APIError::InternalError)?
        .ok_or_else(|| APIError::NotAuthenticated)?;

    require_permission!(permissions, UserPermission::UserAnyRead);


    // Load all users from the database and parse them info `UserInformation` instances.
    let all_users = query::UsersQuery::get_all_users(&state.database)
        .await
        .map_err(APIError::InternalError)?;

    let all_users_as_public_struct: Vec<UserInformation> = all_users
        .into_iter()
        .map(UserInformation::from_user_model)
        .collect();


    Ok(RegisteredUsersListResponse {
        users: all_users_as_public_struct,
    }
    .into_response())
}



/*
 * POST /users
 */

/// User registration request.
#[derive(Deserialize, Clone, Debug, ToSchema)]
pub struct UserRegistrationRequest {
    /// Username to register as (not the same as the display name).
    pub username: String,

    /// Name to display in the UI as.
    pub display_name: String,

    /// Password for this user account.
    pub password: String,
}

/// Conversion into the backend-specific struct for registration
/// (`database::mutation::users::UserRegistrationInfo`).
impl From<UserRegistrationRequest> for UserRegistrationInfo {
    fn from(value: UserRegistrationRequest) -> Self {
        Self {
            username: value.username,
            display_name: value.display_name,
            password: value.password,
        }
    }
}

/// Response on successful user registration.
/// Contains the newly-created user information.
#[derive(Serialize, Debug, ToSchema)]
pub struct UserRegistrationResponse {
    pub user: UserInformation,
}

impl_json_responder!(UserRegistrationResponse);


/// Register a new user
///
/// This endpoint registers a new user with the provided username, display name and password.
/// Only one user with the given username or display name can exist (both fields are required to be unique).
///
/// No authentication is required.
#[utoipa::path(
    post,
    path = "/users",
    tag = "users",
    request_body(
        content = inline(UserRegistrationRequest),
        example = json!({
            "username": "janeznovak",
            "display_name": "Janez Novak",
            "password": "perica_reže_raci_rep"
        })
    ),
    responses(
        (
            status = 200,
            description = "Registration successful.",
            body = inline(UserRegistrationResponse),
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
            status = 409,
            description = "User with given username already exists.",
            body = ErrorReasonResponse,
            examples(
                ("User with same username exists" = (
                    summary = "The username is taken.",
                    value = json!({ "reason": "User with provided username already exists." })
                )),
                ("User with same display name exists" = (
                    summary = "The display name is taken.",
                    value = json!({ "reason": "User with provided display name already exists." })
                )),
            )
        ),
        (
            status = 500,
            description = "Internal server error."
        )
    )
)]
#[post("")]
pub async fn register_user(
    state: web::Data<AppState>,
    json_data: web::Json<UserRegistrationRequest>,
) -> EndpointResult {
    // Ensure the provided username is unique.
    let username_already_exists =
        query::UsersQuery::user_exists_by_username(&state.database, &json_data.username)
            .await
            .map_err(APIError::InternalError)?;

    if username_already_exists {
        return Ok(response_with_reason!(
            StatusCode::CONFLICT,
            "User with provided username already exists."
        ));
    }


    // Ensure the provided display name is unique.
    let display_name_already_exists =
        query::UsersQuery::user_exists_by_display_name(&state.database, &json_data.display_name)
            .await
            .map_err(APIError::InternalError)?;

    if display_name_already_exists {
        return Ok(response_with_reason!(
            StatusCode::CONFLICT,
            "User with provided display name already exists."
        ));
    }


    // Create new user.
    let new_user = mutation::UsersMutation::create_user(
        &state.database,
        &state.hasher,
        json_data.clone().into(),
    )
    .await
    .map_err(APIError::InternalError)?;

    Ok(UserRegistrationResponse {
        user: UserInformation::from_user_model(new_user),
    }
    .into_response())
}



/*
 * GET /users/me
 */

/// Get current user's information
///
/// *This endpoint requires the `users.self:read` permission.*
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
pub async fn get_current_user_info(
    user_auth: UserAuth,
    state: web::Data<AppState>,
) -> EndpointResult {
    // User must provide an authentication token and
    // have the `user.self:read` permission to access this endpoint.
    let (token, permissions) = user_auth
        .token_and_permissions_if_authenticated(&state.database)
        .await
        .map_err(APIError::InternalError)?
        .ok_or_else(|| APIError::NotAuthenticated)?;

    require_permission!(permissions, UserPermission::UserSelfRead);


    // Load user from database.
    let user = query::UsersQuery::get_user_by_username(&state.database, &token.username)
        .await
        .map_err(APIError::InternalError)?
        .ok_or_else(APIError::not_found)?;


    Ok(UserInfoResponse::new(user).into_response())
}



/*
 * GET /users/me/permissions
 */

/// Get current user's permissions
///
/// *This endpoint requires the `users.self:read` permission.*
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
    user_auth: UserAuth,
    state: web::Data<AppState>,
) -> EndpointResult {
    // User must be authenticated and
    // have the `user.self:read` permission to access this endpoint.
    let permissions = user_auth
        .permissions_if_authenticated(&state.database)
        .await
        .map_err(APIError::InternalError)?
        .ok_or_else(|| APIError::NotAuthenticated)?;

    require_permission!(permissions, UserPermission::UserSelfRead);


    Ok(UserPermissionsResponse {
        permissions: permissions.to_permission_names(),
    }
    .into_response())
}



/*
 * PATCH /users/me/display_name
 */

/// Change current user's display name
///
/// This endpoint allows you to change your display name. Note that the display name
/// must be unique across all of the users, so your request may be denied with a `409 Conflict`.
///
/// *This endpoint requires the `users.self:write` permission.*
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
    user_auth: UserAuth,
    state: web::Data<AppState>,
    json_data: web::Json<UserDisplayNameChangeRequest>,
) -> EndpointResult {
    // User must be authenticated and
    // have the `user.self:write` permission to access this endpoint.
    let (token, permissions) = user_auth
        .token_and_permissions_if_authenticated(&state.database)
        .await
        .map_err(APIError::InternalError)?
        .ok_or_else(|| APIError::NotAuthenticated)?;

    require_permission!(permissions, UserPermission::UserSelfWrite);


    let json_data = json_data.into_inner();
    let database_transaction = state
        .database
        .begin()
        .await
        .map_err(APIError::InternalDatabaseError)?;


    // Ensure the display name is unique.
    let display_name_already_exists = query::UsersQuery::user_exists_by_display_name(
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
    mutation::UsersMutation::update_display_name_by_username(
        &database_transaction,
        &token.username,
        json_data.new_display_name.clone(),
    )
    .await
    .map_err(APIError::InternalError)?;

    // TODO Consider merging this update into all mutation methods where it makes sense.
    //      Otherwise we're wasting a round-trip to the database for no real reason.
    let updated_user = mutation::UsersMutation::update_last_active_at_by_username(
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



/*
 * GET /users/{user_id}
 */

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
    user_auth: UserAuth,
    path_info: web::Path<(i32,)>,
    state: web::Data<AppState>,
) -> EndpointResult {
    // Only authenticated users with the `user.any:read` permission can access this endpoint.
    let permissions = user_auth
        .permissions_if_authenticated(&state.database)
        .await
        .map_err(APIError::InternalError)?
        .ok_or_else(|| APIError::NotAuthenticated)?;

    require_permission!(permissions, UserPermission::UserAnyRead);


    // Return information about the requested user.
    let requested_user_id = path_info.into_inner().0;

    let optional_requested_user =
        query::UsersQuery::get_user_by_id(&state.database, requested_user_id)
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
    user_auth: UserAuth,
    path_info: web::Path<(i32,)>,
    state: web::Data<AppState>,
) -> EndpointResult {
    // Only authenticated users with the `user.any:read` permission can access this endpoint.
    let permissions = user_auth
        .permissions_if_authenticated(&state.database)
        .await
        .map_err(APIError::InternalError)?
        .ok_or_else(|| APIError::NotAuthenticated)?;

    require_permission!(permissions, UserPermission::UserAnyRead);


    // Get requested user's permissions.
    let requested_user_id = path_info.into_inner().0;

    let optional_user_permissions =
        query::UserPermissionsQuery::get_user_permission_names_by_user_id(
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
    user_auth: UserAuth,
    path_info: web::Path<(i32,)>,
    state: web::Data<AppState>,
    json_data: web::Json<UserDisplayNameChangeRequest>,
) -> EndpointResult {
    // Only authenticated users with the `user.any:write` permission can modify
    // others' display names. Intended for moderation tooling.
    let (token, permissions) = user_auth
        .token_and_permissions_if_authenticated(&state.database)
        .await
        .map_err(APIError::InternalError)?
        .ok_or_else(|| APIError::NotAuthenticated)?;

    require_permission!(permissions, UserPermission::UserAnyWrite);


    // Disallow modifying your own account on these `/{user_id}/*` endpoints.
    let requested_user_id = path_info.into_inner().0;

    let current_user = query::UsersQuery::get_user_by_username(&state.database, &token.username)
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
    let display_name_already_exists = query::UsersQuery::user_exists_by_display_name(
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
    mutation::UsersMutation::update_display_name_by_user_id(
        &database_transaction,
        requested_user_id,
        json_data.new_display_name.clone(),
    )
    .await
    .map_err(APIError::InternalError)?;

    let updated_user = mutation::UsersMutation::update_last_active_at_by_user_id(
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
    user_auth: UserAuth,
    path_info: web::Path<(i32,)>,
    state: web::Data<AppState>,
    json_data: web::Json<UserPermissionsAddRequest>,
) -> EndpointResult {
    // Only authenticated users with the `user.any:write` permission can add permissions
    // to other users, but only if they also have the requested permission.
    // Intended for moderation tooling.
    let (current_user_token, current_user_permissions) = user_auth
        .token_and_permissions_if_authenticated(&state.database)
        .await
        .map_err(APIError::InternalError)?
        .ok_or_else(|| APIError::NotAuthenticated)?;

    require_permission!(
        current_user_permissions,
        UserPermission::UserAnyWrite
    );


    let requested_user_id = path_info.into_inner().0;
    let json_data = json_data.into_inner();


    // Disallow modifying your own account on these `/{user_id}/*` endpoints.
    let current_user =
        query::UsersQuery::get_user_by_username(&state.database, &current_user_token.username)
            .await
            .map_err(APIError::InternalError)?
            .ok_or_else(|| APIError::internal_reason("BUG: Current user does not exist."))?;

    if current_user.id == requested_user_id {
        return Ok(response_with_reason!(
            StatusCode::FORBIDDEN,
            "Can't modify your own account on this endpoint."
        ));
    }


    let permissions_to_add_result: Result<Vec<UserPermission>, &str> = json_data
        .permissions_to_add
        .iter()
        .map(|permission_name| {
            UserPermission::from_name(permission_name.as_str()).ok_or(permission_name.as_str())
        })
        .collect::<Result<Vec<UserPermission>, &str>>();

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
                    permission.to_name()
                )
            ));
        }
    }


    // Add the permissions to the specified user.
    mutation::UserPermissionsMutation::add_permissions_to_user_by_user_id(
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
    user_auth: UserAuth,
    path_info: web::Path<(i32,)>,
    state: web::Data<AppState>,
    json_data: web::Json<UserPermissionsRemoveRequest>,
) -> EndpointResult {
    // Only authenticated users with the `user.any:write` permission can remove permissions
    // from other users, but not those that they themselves don't have.
    // Intended for moderation tooling.
    let (current_user_token, current_user_permissions) = user_auth
        .token_and_permissions_if_authenticated(&state.database)
        .await
        .map_err(APIError::InternalError)?
        .ok_or_else(|| APIError::NotAuthenticated)?;

    require_permission!(
        current_user_permissions,
        UserPermission::UserAnyWrite
    );


    let requested_user_id = path_info.into_inner().0;
    let json_data = json_data.into_inner();


    // Disallow modifying your own account on these `/{user_id}/*` endpoints.
    let current_user =
        query::UsersQuery::get_user_by_username(&state.database, &current_user_token.username)
            .await
            .map_err(APIError::InternalError)?
            .ok_or_else(|| APIError::internal_reason("BUG: Current user does not exist."))?;

    if current_user.id == requested_user_id {
        return Ok(response_with_reason!(
            StatusCode::FORBIDDEN,
            "Can't modify your own account on this endpoint."
        ));
    }


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
    mutation::UserPermissionsMutation::remove_permissions_from_user_by_user_id(
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
        permissions: updated_permission_list.to_permission_names(),
    }
    .into_response())
}



/*
 * Router
 */

pub fn users_router() -> Scope {
    web::scope("users")
        .service(get_all_registered_users)
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
