use actix_web::{post, web};
use kolomoni_core::api_models::{UserRegistrationRequest, UserRegistrationResponse};
use kolomoni_database::entities::{self, UserRegistrationInfo};
use sqlx::Acquire;

use crate::{
    api::{
        errors::{EndpointResponseBuilder, EndpointResult, UsersErrorReason},
        openapi::{self, response::AsErrorReason},
        traits::IntoApiModel,
    },
    declare_openapi_error_reason_response,
    state::ApplicationState,
};



declare_openapi_error_reason_response!(
    pub struct RegistrationUsernameIsTaken {
        description => "The provided username is already in use.",
        reason => UsersErrorReason::username_already_exists()
    }
);

declare_openapi_error_reason_response!(
    pub struct RegistrationDisplayNameIsTaken {
        description => "The provided display name is already in use.",
        reason => UsersErrorReason::display_name_already_exists()
    }
);


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
            response = inline(AsErrorReason<RegistrationUsernameIsTaken>)
        ),
        (
            status = 409,
            response = inline(AsErrorReason<RegistrationDisplayNameIsTaken>)
        ),
        openapi::response::RequiredJsonBodyErrors,
        openapi::response::InternalServerError,
    )
)]
#[post("")]
pub async fn register_user(
    state: ApplicationState,
    request_data: web::Json<UserRegistrationRequest>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;
    let mut transaction = database_connection.begin().await?;


    let registration_request_data = request_data.into_inner();


    // Ensure the provided username is unique.
    let username_already_exists = entities::UserQuery::exists_by_username(
        &mut transaction,
        &registration_request_data.username,
    )
    .await?;

    if username_already_exists {
        return EndpointResponseBuilder::conflict()
            .with_error_reason(UsersErrorReason::username_already_exists())
            .build();
    }


    // Ensure the provided display name is unique.
    let display_name_already_exists = entities::UserQuery::exists_by_display_name(
        &mut transaction,
        &registration_request_data.display_name,
    )
    .await?;

    if display_name_already_exists {
        return EndpointResponseBuilder::conflict()
            .with_error_reason(UsersErrorReason::display_name_already_exists())
            .build();
    }


    // Create new user.
    let newly_created_user = entities::UserMutation::create_user(
        &mut transaction,
        state.hasher(),
        UserRegistrationInfo {
            username: registration_request_data.username,
            display_name: registration_request_data.display_name,
            password: registration_request_data.password,
        },
    )
    .await?;


    transaction.commit().await?;


    EndpointResponseBuilder::ok()
        .with_json_body(UserRegistrationResponse {
            user: newly_created_user.into_api_model(),
        })
        .build()
}
