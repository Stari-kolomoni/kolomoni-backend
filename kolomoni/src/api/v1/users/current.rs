use actix_web::{
    get,
    http::{header, StatusCode},
    patch,
    web,
    HttpResponse,
};
use kolomoni_auth::Permission;
use kolomoni_database::{
    mutation,
    query::{self, UserQuery, UserRoleQuery},
};
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
        openapi,
        v1::users::{
            UserDisplayNameChangeRequest,
            UserDisplayNameChangeResponse,
            UserInfoResponse,
            UserInformation,
            UserPermissionsResponse,
            UserRolesResponse,
        },
        OptionalIfModifiedSince,
    },
    authentication::UserAuthenticationExtractor,
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
    if_modified_since: OptionalIfModifiedSince,
) -> EndpointResult {
    // User must provide an authentication token and
    // have the `user.self:read` permission to access this endpoint.
    let authenticated_user = require_authentication!(authentication_extractor);
    let authenticated_user_id = authenticated_user.user_id();
    require_permission!(
        state,
        authenticated_user,
        Permission::UserSelfRead
    );


    // Load user from database.
    let user = query::UserQuery::get_user_by_id(&state.database, authenticated_user_id)
        .await
        .map_err(APIError::InternalError)?
        .ok_or_else(APIError::not_found)?;

    let last_modification_time = user.last_modified_at.to_utc();

    if if_modified_since.has_not_changed_since(&last_modification_time) {
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


#[utoipa::path(
    get,
    path = "/users/me/roles",
    responses(
        (
            status = 200,
            description = "The authenticated user's role list.",
            body = UpdatedUserRolesResponse
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
    let authenticated_user = require_authentication!(authentication);
    require_permission!(
        state,
        authenticated_user,
        Permission::UserSelfRead
    );

    let user_exists =
        UserQuery::user_exists_by_user_id(&state.database, authenticated_user.user_id())
            .await
            .map_err(APIError::InternalError)?;
    if !user_exists {
        return Err(APIError::not_found());
    }


    let user_roles = UserRoleQuery::user_roles(&state.database, authenticated_user.user_id())
        .await
        .map_err(APIError::InternalError)?;

    let user_role_names = user_roles.role_names();

    Ok(UserRolesResponse {
        role_names: user_role_names,
    }
    .into_response())
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
    // User must be authenticated and
    // have the `user.self:read` permission to access this endpoint.
    let authenticated_user = require_authentication!(authentication_extractor);
    let user_permissions = authenticated_user
        .permissions(&state.database)
        .await
        .map_err(APIError::InternalError)?;

    require_permission!(user_permissions, Permission::UserSelfRead);


    let permission_names = user_permissions
        .permission_names()
        .into_iter()
        .map(|name_static_str| name_static_str.to_string())
        .collect();

    Ok(UserPermissionsResponse::from_permission_names(permission_names).into_response())
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
            status = 409,
            description = "User with given display name already exists.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "User with given display name already exists." })
        ),
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
    // User must be authenticated and have
    // the `user.self:write` permission to access this endpoint.
    let authenticated_user = require_authentication!(authentication_extractor);
    let authenticated_user_id = authenticated_user.user_id();
    require_permission!(
        state,
        authenticated_user,
        Permission::UserSelfWrite
    );


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
    let updated_user = mutation::UserMutation::update_display_name_by_user_id(
        &database_transaction,
        authenticated_user_id,
        json_data.new_display_name.clone(),
    )
    .await
    .map_err(APIError::InternalError)?;

    database_transaction
        .commit()
        .await
        .map_err(APIError::InternalDatabaseError)?;


    info!(
        user_id = authenticated_user_id,
        new_display_name = json_data.new_display_name,
        "User has updated their display name."
    );

    Ok(UserDisplayNameChangeResponse {
        user: UserInformation::from_user_model(updated_user),
    }
    .into_response())
}
