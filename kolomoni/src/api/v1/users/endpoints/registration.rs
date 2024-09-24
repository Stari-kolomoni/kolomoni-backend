use actix_web::{http::StatusCode, post, web};
use kolomoni_core::api_models::{UserRegistrationRequest, UserRegistrationResponse};
use kolomoni_database::entities::{self, UserRegistrationInfo};
use sqlx::Acquire;

use crate::{
    api::{errors::EndpointResult, macros::ContextlessResponder, openapi, traits::IntoApiModel},
    json_error_response_with_reason,
    obtain_database_connection,
    state::ApplicationState,
};




/// Register a new user
///
/// This endpoint registers a new user with the provided username, display name and password.
///
/// Both the username and the display name must be unique across all users,
/// i.e. no two users can share the same username or display name.
///
/// # Authentication
/// This endpoint does not require authentication.
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
        openapi::response::MissingOrInvalidJsonRequestBody,
        openapi::response::InternalServerError,
    )
)]
#[post("")]
pub async fn register_user(
    state: ApplicationState,
    request_data: web::Json<UserRegistrationRequest>,
) -> EndpointResult {
    let mut database_connection = obtain_database_connection!(state);
    let mut transaction = database_connection.begin().await?;


    let registration_request_data = request_data.into_inner();


    // Ensure the provided username is unique.
    let username_already_exists = entities::UserQuery::exists_by_username(
        &mut transaction,
        &registration_request_data.username,
    )
    .await?;

    if username_already_exists {
        return Ok(json_error_response_with_reason!(
            StatusCode::CONFLICT,
            "User with provided username already exists."
        ));
    }


    // Ensure the provided display name is unique.
    let display_name_already_exists = entities::UserQuery::exists_by_display_name(
        &mut transaction,
        &registration_request_data.display_name,
    )
    .await?;

    if display_name_already_exists {
        return Ok(json_error_response_with_reason!(
            StatusCode::CONFLICT,
            "User with provided display name already exists."
        ));
    }


    // Create new user.
    let newly_created_user = entities::UserMutation::create_user(
        &mut transaction,
        &state.hasher,
        UserRegistrationInfo {
            username: registration_request_data.username,
            display_name: registration_request_data.display_name,
            password: registration_request_data.password,
        },
    )
    .await?;


    transaction.commit().await?;


    Ok(UserRegistrationResponse {
        user: newly_created_user.into_api_model(),
    }
    .into_response())
}
