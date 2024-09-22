use actix_web::get;
use futures_util::StreamExt;
use kolomoni_auth::Permission;
use kolomoni_core::api_models::RegisteredUsersListResponse;
use kolomoni_database::entities;

use crate::{
    api::{errors::EndpointResult, macros::ContextlessResponder, openapi, traits::IntoApiModel},
    authentication::UserAuthenticationExtractor,
    impl_json_response_builder,
    obtain_database_connection,
    require_authentication,
    require_permission,
    state::ApplicationState,
};


impl_json_response_builder!(RegisteredUsersListResponse);



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
    let mut database_connection = obtain_database_connection!(state);


    // To access this endpoint, the user:
    // - MUST provide their authentication token, and
    // - MUST have the `user.any:read` permission.
    let authenticated_user = require_authentication!(authentication);
    require_permission!(
        &mut database_connection,
        authenticated_user,
        Permission::UserAnyRead
    );



    // Load all users from the database and parse them info `UserInformation` instances.
    let mut all_users_stream = entities::UserQuery::get_all_users(&mut database_connection);


    let mut parsed_users = Vec::new();

    while let Some(next_user_result) = all_users_stream.next().await {
        let next_user_as_api_model = next_user_result?.into_api_model();

        parsed_users.push(next_user_as_api_model);
    }


    Ok(RegisteredUsersListResponse {
        users: parsed_users,
    }
    .into_response())
}
