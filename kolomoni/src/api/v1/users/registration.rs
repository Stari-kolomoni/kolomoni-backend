use actix_web::{http::StatusCode, post, web};
use kolomoni_database::{
    mutation::{self, UserRegistrationInfo},
    query,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::UserInformation;
use crate::{
    api::{
        errors::{APIError, EndpointResult},
        macros::ContextlessResponder,
        openapi,
    },
    error_response_with_reason,
    impl_json_response_builder,
    state::ApplicationState,
};

/// User registration request provided by an API caller.
#[derive(Deserialize, Clone, Debug, ToSchema)]
#[schema(example = json!({
    "username": "janeznovak",
    "display_name": "Janez Novak",
    "password": "perica_reže_raci_rep"
}))]
#[cfg_attr(feature = "with_test_facilities", derive(Serialize))]
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

/// API-serializable response upon successful user registration.
/// Contains the newly-created user's information.
#[derive(Serialize, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Deserialize))]
#[schema(
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
)]
pub struct UserRegistrationResponse {
    pub user: UserInformation,
}

impl_json_response_builder!(UserRegistrationResponse);



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
        content = UserRegistrationRequest
    ),
    responses(
        (
            status = 200,
            description = "Registration successful.",
            body = UserRegistrationResponse
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
        openapi::InternalServerErrorResponse,
    )
)]
#[post("")]
pub async fn register_user(
    state: ApplicationState,
    json_data: web::Json<UserRegistrationRequest>,
) -> EndpointResult {
    // Ensure the provided username is unique.
    let username_already_exists =
        query::UserQuery::user_exists_by_username(&state.database, &json_data.username)
            .await
            .map_err(APIError::InternalError)?;

    if username_already_exists {
        return Ok(error_response_with_reason!(
            StatusCode::CONFLICT,
            "User with provided username already exists."
        ));
    }


    // Ensure the provided display name is unique.
    let display_name_already_exists =
        query::UserQuery::user_exists_by_display_name(&state.database, &json_data.display_name)
            .await
            .map_err(APIError::InternalError)?;

    if display_name_already_exists {
        return Ok(error_response_with_reason!(
            StatusCode::CONFLICT,
            "User with provided display name already exists."
        ));
    }


    // Create new user.
    let new_user = mutation::UserMutation::create_user(
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
