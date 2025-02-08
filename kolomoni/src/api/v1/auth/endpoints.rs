use actix_web::{post, web};
use chrono::{Duration, Utc};
use kolomoni_core::api_models::{
    LoginErrorReason,
    UserLoginRefreshRequest,
    UserLoginRefreshResponse,
    UserLoginRequest,
    UserLoginResponse,
    UserRegistrationRequest,
    UserRegistrationResponse,
    UsersErrorReason,
};
use kolomoni_core::token::{JWTClaims, JWTTokenType, JWTValidationError};
use kolomoni_database::entities::user::{UserMutation, UserQuery, UserRegistrationInfo};
use tracing::{debug, warn};

use crate::api::errors::{EndpointResponseBuilder, EndpointResult};
use crate::api::openapi;
use crate::api::openapi::response::AsErrorReason;
use crate::api::traits::IntoApiModel;
use crate::declare_openapi_error_reason_response;
use crate::state::ApplicationState;


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
    path = "/auth/register",
    tag = "authentication",
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
#[post("/register")]
pub async fn register_user(
    state: ApplicationState,
    request_data: web::Json<UserRegistrationRequest>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;
    let mut transaction = database_connection.transaction().begin().await?;


    let registration_request_data = request_data.into_inner();


    // Ensure the provided username is unique.
    let username_already_exists = UserQuery::exists_by_username(
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
    let display_name_already_exists = UserQuery::exists_by_display_name(
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
    let newly_created_user = UserMutation::create_user(
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



declare_openapi_error_reason_response!(
    pub struct LoginInvalidCredentials {
        description => "Invalid login credentials.",
        reason => LoginErrorReason::invalid_login_credentials()
    }
);



/// Login
///
/// This endpoint is the login method: it validates the credentials (username and password) and
/// gives the user an access token they can use in future requests to authenticate themselves.
///
/// In addition to the access token, a refresh token is provided to the user so they can request
/// a new access token when it expires. The refresh token is valid for longer than the access token,
/// but only the access token can be used in the *Authorization* header.
///
/// For login refreshing, see the `POST /api/v1/login/refresh` endpoint.
#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "authentication",
    request_body(
        content = UserLoginRequest
    ),
    responses(
        (
            status = 200,
            description = "Login successful.",
            body = UserLoginResponse
        ),
        (
            status = 403,
            response = inline(AsErrorReason<LoginInvalidCredentials>)
        ),
        openapi::response::RequiredJsonBodyErrors,
        openapi::response::InternalServerError,
    )
)]
#[post("/login")]
pub async fn login(
    state: ApplicationState,
    login_info: web::Json<UserLoginRequest>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;


    // Validate user login credentials.
    let login_result = UserQuery::validate_credentials(
        &mut database_connection,
        state.hasher(),
        &login_info.username,
        &login_info.password,
    )
    .await?;

    let Some(logged_in_user) = login_result else {
        return EndpointResponseBuilder::forbidden()
            .with_error_reason(LoginErrorReason::invalid_login_credentials())
            .build();
    };


    // Generate access and refresh token.
    let logged_in_at = Utc::now();


    // TODO Make this configurable.

    let access_token_claims = JWTClaims::create(
        logged_in_user.id,
        logged_in_at,
        Duration::hours(2),
        JWTTokenType::Access,
    );

    let refresh_token_claims = JWTClaims::create(
        logged_in_user.id,
        logged_in_at,
        Duration::days(7),
        JWTTokenType::Refresh,
    );


    let access_token = state.jwt_manager().create_token(access_token_claims)?;
    let refresh_token = state.jwt_manager().create_token(refresh_token_claims)?;


    debug!(
        username = login_info.username,
        "User has successfully logged in."
    );


    EndpointResponseBuilder::ok()
        .with_json_body(UserLoginResponse {
            access_token,
            refresh_token,
        })
        .build()
}



declare_openapi_error_reason_response!(
    pub struct LoginRefreshTokenHasExpired {
        description => "The provided refresh token has expired.",
        reason => LoginErrorReason::expired_refresh_token()
    }
);

declare_openapi_error_reason_response!(
    pub struct LoginInvalidRefreshToken {
        description => "The provided refresh token is invalid (not a valid JWT).",
        reason => LoginErrorReason::invalid_refresh_json_web_token()
    }
);

declare_openapi_error_reason_response!(
    pub struct LoginNotARefreshToken {
        description => "The provided JWT is not a refresh token.",
        reason => LoginErrorReason::not_a_refresh_token()
    }
);


/// Refresh a login
///
/// The user must provide a refresh token given to them on an initial call to `/users/login`.
/// "Refreshing a login" does not invalidate the refresh token.
///
/// The result of this is essentially a new JWT access token. Use when your initial access token
/// from `/users/login` expires.
#[utoipa::path(
    post,
    path = "/auth/refresh",
    tag = "authentication",
    request_body(
        content = UserLoginRefreshRequest,
    ),
    responses(
        (
            status = 200,
            description = "Login refresh successful.",
            body = UserLoginRefreshResponse
        ),
        (
            status = 400,
            response = inline(AsErrorReason<LoginRefreshTokenHasExpired>)
        ),
        (
            status = 400,
            response = inline(AsErrorReason<LoginInvalidRefreshToken>)
        ),
        (
            status = 400,
            response = inline(AsErrorReason<LoginNotARefreshToken>)
        ),
        openapi::response::RequiredJsonBodyErrors,
        openapi::response::InternalServerError,
    )
)]
#[post("/refresh")]
pub async fn refresh_login(
    state: ApplicationState,
    refresh_info: web::Json<UserLoginRefreshRequest>,
) -> EndpointResult {
    // Parse and validate provided refresh token.
    let refresh_token_claims = match state
        .jwt_manager()
        .decode_token(&refresh_info.refresh_token)
    {
        Ok(token_claims) => token_claims,
        Err(error) => {
            return match error {
                JWTValidationError::Expired { expired_token } => {
                    debug!(
                        user_id = %expired_token.user_id,
                        "Refusing to refresh expired token.",
                    );

                    EndpointResponseBuilder::bad_request()
                        .with_error_reason(LoginErrorReason::expired_refresh_token())
                        .build()
                }
                JWTValidationError::InvalidToken { reason } => {
                    warn!(error = %reason, "Failed to parse refresh token.");

                    EndpointResponseBuilder::bad_request()
                        .with_error_reason(LoginErrorReason::invalid_refresh_json_web_token())
                        .build()
                }
            };
        }
    };

    if refresh_token_claims.token_type != JWTTokenType::Refresh {
        return EndpointResponseBuilder::bad_request()
            .with_error_reason(LoginErrorReason::not_a_refresh_token())
            .build();
    }

    // Refresh token is valid, create new access token.
    let access_token_claims = JWTClaims::create(
        refresh_token_claims.user_id,
        Utc::now(),
        Duration::days(1),
        JWTTokenType::Access,
    );

    let access_token = state.jwt_manager().create_token(access_token_claims)?;


    debug!(
        user_id = %refresh_token_claims.user_id,
        "User has successfully refreshed access token."
    );


    EndpointResponseBuilder::ok()
        .with_json_body(UserLoginRefreshResponse { access_token })
        .build()
}
