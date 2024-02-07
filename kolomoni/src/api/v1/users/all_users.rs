use actix_web::get;
use kolomoni_auth::Permission;
use kolomoni_database::query;
use serde::Serialize;
use utoipa::ToSchema;

use super::UserInformation;
use crate::{
    api::{
        errors::{APIError, EndpointResult},
        macros::DumbResponder,
    },
    authentication::UserAuth,
    impl_json_responder,
    require_authentication,
    require_permission,
    state::ApplicationState,
};

/// List of registered users.
#[derive(Serialize, Debug, ToSchema)]
pub struct RegisteredUsersListResponse {
    pub users: Vec<UserInformation>,
}

impl_json_responder!(RegisteredUsersListResponse);


/// List all registered users.
///
/// This endpoint returns a list of all registered users.
///
///
/// # Required permissions
/// This endpoint requires the `users.any:read` permission.
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
async fn get_all_registered_users(state: ApplicationState, user_auth: UserAuth) -> EndpointResult {
    // User must provide the authentication token and
    // have the `user.any:read` permission to access this endpoint.
    let (_, permissions) = require_authentication!(state, user_auth);
    require_permission!(permissions, Permission::UserAnyRead);


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
