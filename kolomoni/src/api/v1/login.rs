use actix_web::body::BoxBody;
use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use chrono::{Duration, Utc};
use kolomoni_auth::token::{JWTClaims, JWTTokenType, JWTValidationError};
use kolomoni_database::query;
use miette::Context;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};
use utoipa::ToSchema;

use crate::api::errors::{APIError, EndpointResult, ErrorReasonResponse};
use crate::api::macros::DumbResponder;
use crate::impl_json_responder;
use crate::state::AppState;

/*
 * POST /login
 */

/// User login information.
#[derive(Deserialize, Debug, ToSchema)]
pub struct UserLoginRequest {
    /// Username to log in as.
    pub username: String,

    /// Password.
    pub password: String,
}

/// Response on successful user login.
///
/// Contains two tokens:
/// - the `access_token` that should be appended to future requests and
/// - the `refresh_token` that can be used on `POST /api/v1/users/login/refresh` to
///   receive a new (fresh) request token.
///
/// This works because the `refresh_token` has a longer expiration time.
#[derive(Serialize, Debug, ToSchema)]
pub struct UserLoginResponse {
    /// JWT access token.
    pub access_token: String,

    /// JWT refresh token.
    pub refresh_token: String,
}

impl_json_responder!(UserLoginResponse);


/// Validate provided user credentials and generate access token
///
/// This endpoint validates the credentials (username and password) and gives the user
/// an access token they can use in future requests to authenticate themselves.
///
/// A refresh token is also provided to the user can request a new access token (the refresh
/// token is valid for longer than the access token, but only the access token can be added
/// in the *Authorization* header).
#[utoipa::path(
    post,
    path = "/login",
    tag = "login",
    request_body(
        content = inline(UserLoginRequest),
        example = json!({ "username": "sample_user", "password": "verysecurepassword" })
    ),
    responses(
        (
            status = 200,
            description = "Login successful.",
            body = inline(UserLoginResponse),
            example = json!({
                "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJTdGFyaSBLb2xvbW9uaSIsInN1YiI6IkFQSSB0b2tlbiIsImlhdCI6MTY4Nzk3MTMyMiwiZXhwIjoxNjg4MDU3NzIyLCJ1c2VybmFtZSI6InRlc3QiLCJ0b2tlbl90eXBlIjoiYWNjZXNzIn0.ZnuhEVacQD_pYzkW9h6aX3eoRNOAs2-y3EngGBglxkk",
                "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJTdGFyaSBLb2xvbW9uaSIsInN1YiI6IkFQSSB0b2tlbiIsImlhdCI6MTY4Nzk3MTMyMiwiZXhwIjoxNjg4NTc2MTIyLCJ1c2VybmFtZSI6InRlc3QiLCJ0b2tlbl90eXBlIjoicmVmcmVzaCJ9.Ze6DI5EZ-swXRQrMW3NIppYejclGbyI9D6zmYBWJMLk"
            })
        ),
        (
            status = 403,
            description = "Invalid login information.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "Invalid login credentials." })
        ),
        (
            status = 500,
            description = "Internal server error."
        )
    )
)]
#[post("/login")]
pub async fn login(
    state: web::Data<AppState>,
    login_info: web::Json<UserLoginRequest>,
) -> EndpointResult {
    // Validate user login credentials.
    let is_valid_login = query::UsersQuery::validate_user_credentials(
        &state.database,
        &state.hasher,
        &login_info.username,
        &login_info.password,
    )
    .await
    .map_err(APIError::InternalError)?;

    if !is_valid_login {
        return Ok(
            HttpResponse::Forbidden().json(ErrorReasonResponse::custom_reason(
                "Invalid login credentials.",
            )),
        );
    }


    // Generate access and refresh token.
    let access_token_claims = JWTClaims::create(
        login_info.username.clone(),
        Utc::now(),
        Duration::days(1),
        JWTTokenType::Access,
    );

    let refresh_token_claims = JWTClaims::create(
        login_info.username.clone(),
        Utc::now(),
        Duration::days(7),
        JWTTokenType::Refresh,
    );


    let access_token = state
        .jwt_manager
        .create_token(access_token_claims)
        .wrap_err("Errored while creating JWT access token.")
        .map_err(APIError::InternalError)?;

    let refresh_token = state
        .jwt_manager
        .create_token(refresh_token_claims)
        .wrap_err("Errored while creating JWT refresh token.")
        .map_err(APIError::InternalError)?;

    debug!(
        username = login_info.username,
        "User has successfully logged in."
    );

    Ok(UserLoginResponse {
        access_token,
        refresh_token,
    }
    .into_response())
}



/*
 * POST /login/refresh
 */

/// Information with which to refresh a user's login, generating a new access token.
#[derive(Deserialize, ToSchema)]
pub struct UserLoginRefreshRequest {
    /// Refresh token to use to generate an access token.
    ///
    /// Token must not have expired to work.
    pub refresh_token: String,
}

/// Response on successful login refresh.
#[derive(Serialize, Debug, ToSchema)]
pub struct UserLoginRefreshResponse {
    /// Newly-generated access token to use in future requests.
    pub access_token: String,
}

impl_json_responder!(UserLoginRefreshResponse);


/// Refresh a user's access
///
/// The user must provide a refresh token given to them on an initial call to `/users/login`.
/// "Refreshing a login" does not invalidate the refresh token.
///
/// The result of this is essentially a new JWT access token. Use when your initial access token
/// from `/users/login` expires.
#[utoipa::path(
    post,
    path = "/login/refresh",
    tag = "login",
    request_body(
        content = inline(UserLoginRefreshRequest),
        example = json!({ "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJTdGFyaSBLb2xvbW9uaSIsInN1YiI6IkFQSSB0b2tlbiIsImlhdCI6MTY4Nzk3MTMyMiwiZXhwIjoxNjg4NTc2MTIyLCJ1c2VybmFtZSI6InRlc3QiLCJ0b2tlbl90eXBlIjoicmVmcmVzaCJ9.Ze6DI5EZ-swXRQrMW3NIppYejclGbyI9D6zmYBWJMLk" })
    ),
    responses(
        (
            status = 200,
            description = "Login refresh successful.",
            body = inline(UserLoginRefreshResponse),
            example = json!({ "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJTdGFyaSBLb2xvbW9uaSIsInN1YiI6IkFQSSB0b2tlbiIsImlhdCI6MTY4Nzk3MTMyMiwiZXhwIjoxNjg4MDU3NzIyLCJ1c2VybmFtZSI6InRlc3QiLCJ0b2tlbl90eXBlIjoiYWNjZXNzIn0.ZnuhEVacQD_pYzkW9h6aX3eoRNOAs2-y3EngGBglxkk" })
        ),
        (
            status = 403,
            description = "Refresh token has expired.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "Refresh token has expired." })
        ),
        (
            status = 400,
            description = "Invalid refresh token.",
            body = ErrorReasonResponse,
            examples(
                ("Invalid JWT token" = (
                    summary = "The provided JWT refresh token is not a valid token at all.",
                    value = json!({ "reason": "Invalid refresh token." })
                )),
                ("Not a refresh token" = (
                    summary = "The provided JWT token is not a refresh token.",
                    value = json!({ "reason": "The provided token is not a refresh token." })
                ))
            )
        ),
        (
            status = 500,
            description = "Internal server error."
        )
    )
)]
#[post("/login/refresh")]
pub async fn refresh_login(
    state: web::Data<AppState>,
    refresh_info: web::Json<UserLoginRefreshRequest>,
) -> EndpointResult {
    // Parse and validate provided refresh token.
    let refresh_token_claims = match state.jwt_manager.decode_token(&refresh_info.refresh_token) {
        Ok(token_claims) => token_claims,
        Err(error) => {
            return match error {
                JWTValidationError::Expired(token_claims) => {
                    debug!(
                        username = token_claims.username,
                        "Refusing to refresh expired token.",
                    );

                    Ok(
                        HttpResponse::Forbidden().json(ErrorReasonResponse::custom_reason(
                            "Refresh token has expired.",
                        )),
                    )
                }
                JWTValidationError::InvalidToken(error) => {
                    warn!(error = error, "Failed to parse refresh token.");

                    Ok(
                        HttpResponse::BadRequest().json(ErrorReasonResponse::custom_reason(
                            "Invalid refresh token.",
                        )),
                    )
                }
            }
        }
    };

    if refresh_token_claims.token_type != JWTTokenType::Refresh {
        return Ok(
            HttpResponse::BadRequest().json(ErrorReasonResponse::custom_reason(
                "The provided token is not a refresh token.",
            )),
        );
    }

    // Refresh token is valid, create new access token.
    let access_token_claims = JWTClaims::create(
        refresh_token_claims.username.clone(),
        Utc::now(),
        Duration::days(1),
        JWTTokenType::Access,
    );
    let access_token = state
        .jwt_manager
        .create_token(access_token_claims)
        .map_err(APIError::InternalError)?;

    debug!(
        username = refresh_token_claims.username,
        "User has successfully refreshed access token."
    );

    Ok(UserLoginRefreshResponse { access_token }.into_response())
}
