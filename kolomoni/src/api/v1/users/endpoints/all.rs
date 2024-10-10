use actix_web::get;
use futures_util::StreamExt;
use kolomoni_core::api_models::RegisteredUsersListResponse;
use kolomoni_core::permissions::Permission;
use kolomoni_database::entities;

use crate::{
    api::{
        errors::{EndpointResponseBuilder, EndpointResult},
        openapi::{self, response::requires},
        traits::IntoApiModel,
    },
    authentication::UserAuthenticationExtractor,
    require_user_authentication_and_permissions,
    state::ApplicationState,
};



/// List all registered users.
///
/// This endpoint returns a list of all registered users.
///
///
/// # Authentication
/// This endpoint requires authentication and the `users.any:read` permission.
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
        openapi::response::MissingAuthentication,
        openapi::response::MissingPermissions<requires::UserAnyRead, 1>,
        openapi::response::InternalServerError,
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
    let mut database_connection = state.acquire_database_connection().await?;


    // To access this endpoint, the user:
    // - MUST provide their authentication token, and
    // - MUST have the `user.any:read` permission.
    require_user_authentication_and_permissions!(
        &mut database_connection,
        authentication,
        Permission::UserAnyRead
    );


    // Load all users from the database and parse each into [`UserInfo`].
    let mut all_users_stream = entities::UserQuery::get_all_users(&mut database_connection);

    let mut parsed_users = Vec::new();
    while let Some(next_user_result) = all_users_stream.next().await {
        parsed_users.push(next_user_result?.into_api_model());
    }


    EndpointResponseBuilder::ok()
        .with_json_body(RegisteredUsersListResponse {
            users: parsed_users,
        })
        .build()
}
