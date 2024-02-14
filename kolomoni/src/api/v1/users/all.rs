use actix_web::get;
use kolomoni_auth::Permission;
use kolomoni_database::query;
use serde::Serialize;
use utoipa::ToSchema;

use super::UserInformation;
use crate::{
    api::{
        errors::{APIError, EndpointResult},
        macros::ContextlessResponder,
        openapi,
    },
    authentication::UserAuthenticationExtractor,
    impl_json_response_builder,
    require_authentication,
    require_permission,
    state::ApplicationState,
};

/// List of registered users.
#[derive(Serialize, Debug, ToSchema)]
#[schema(title = "RegisteredUsersListResponse")]
#[schema(example = json!({
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
}))]
pub struct RegisteredUsersListResponse {
    pub users: Vec<UserInformation>,
}

impl_json_response_builder!(RegisteredUsersListResponse);


/// List all registered users.
///
/// This endpoint returns a list of all registered users.
///
///
/// # Permissions
/// This endpoint requires the `users.any:read` permission.
#[utoipa::path(
    get,
    path = "/users",
    tag = "users",
    responses(
        (
            status = 200,
            description = "List of registered users.",
            body = RegisteredUsersListResponse
        ),
        openapi::FailedAuthenticationResponses<openapi::RequiresUserAnyRead>,
        openapi::InternalServerErrorResponse,
    ),
    security(
        ("access_token" = [])
    )
)]
#[get("")]
pub async fn get_all_registered_users(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
) -> EndpointResult {
    // User must provide their authentication token and
    // have the `user.any:read` permission to access this endpoint.
    let authenticated_user = require_authentication!(authentication);
    require_permission!(state, authenticated_user, Permission::UserAnyRead);


    // Load all users from the database and parse them info `UserInformation` instances.
    let all_users = query::UserQuery::get_all_users(&state.database)
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
