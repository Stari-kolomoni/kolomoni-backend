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
        macros::DumbResponder,
    },
    impl_json_responder,
    response_with_reason,
    state::ApplicationState,
};

/// User registration request provided by an API caller.
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

/// API-serializable response upon successful user registration.
/// Contains the newly-created user's information.
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
    state: ApplicationState,
    json_data: web::Json<UserRegistrationRequest>,
) -> EndpointResult {
    // Ensure the provided username is unique.
    let username_already_exists =
        query::UserQuery::user_exists_by_username(&state.database, &json_data.username)
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
        query::UserQuery::user_exists_by_display_name(&state.database, &json_data.display_name)
            .await
            .map_err(APIError::InternalError)?;

    if display_name_already_exists {
        return Ok(response_with_reason!(
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
