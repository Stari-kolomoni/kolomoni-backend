use actix_web::{post, web, HttpResponse, Scope};
use chrono::{Duration, Utc};
use kolomoni_auth::{JWTClaims, JWTTokenType, JWTValidationError};
use kolomoni_database::query;
use miette::Context;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};
use utoipa::ToSchema;

use crate::api::errors::{APIError, EndpointResult, ErrorReasonResponse};
use crate::api::macros::ContextlessResponder;
use crate::api::openapi;
use crate::impl_json_response_builder;
use crate::state::ApplicationState;



/// User login information.
#[derive(Deserialize, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Serialize))]
#[schema(
    example = json!({
        "username": "sample_user",
        "password": "verysecurepassword" 
    })
)]
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
#[derive(Serialize, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Deserialize))]
#[schema(
    example = json!({
        "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJTdGFyaSBLb2xvbW9uaSIsInN1Y\
                         iI6IkFQSSB0b2tlbiIsImlhdCI6MTY4Nzk3MTMyMiwiZXhwIjoxNjg4MDU3NzIyLCJ1c2VybmF\
                         tZSI6InRlc3QiLCJ0b2tlbl90eXBlIjoiYWNjZXNzIn0.ZnuhEVacQD_pYzkW9h6aX3eoRNOAs\
                         2-y3EngGBglxkk",
        "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJTdGFyaSBLb2xvbW9uaSIsInN1\
                          YiI6IkFQSSB0b2tlbiIsImlhdCI6MTY4Nzk3MTMyMiwiZXhwIjoxNjg4NTc2MTIyLCJ1c2Vyb\
                          mFtZSI6InRlc3QiLCJ0b2tlbl90eXBlIjoicmVmcmVzaCJ9.Ze6DI5EZ-swXRQrMW3NIppYej\
                          clGbyI9D6zmYBWJMLk"
    })
)]
pub struct UserLoginResponse {
    /// JWT access token.
    /// Provide in subsequent requests in the `Authorization` header as `Bearer your_token_here`.
    pub access_token: String,

    /// JWT refresh token.
    pub refresh_token: String,
}

impl_json_response_builder!(UserLoginResponse);



/// Login
///
/// This endpoint is the login method: it validates the credentials (username and password) and
/// gives the user an access token they can use in future requests to authenticate themselves.
///
/// In addition to the access token, a refresh token is provided to the user so they can request
/// a new access token. The refresh token is valid for longer than the access token,
/// but only the access token can be used in the *Authorization* header. For login refreshing,
/// see the `POST /api/v1/login/refresh` endpoint.
#[utoipa::path(
    post,
    path = "/login",
    tag = "login",
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
            description = "Invalid login information.",
            body = ErrorReasonResponse,
            example = json!({ "reason": "Invalid login credentials." })
        ),
        openapi::MissingOrInvalidJsonRequestBodyResponse,
        openapi::InternalServerErrorResponse,
    )
)]
#[post("")]
pub async fn login(
    state: ApplicationState,
    login_info: web::Json<UserLoginRequest>,
) -> EndpointResult {
    // Validate user login credentials.
    let login_result_details = query::UserQuery::validate_user_credentials(
        &state.database,
        &state.hasher,
        &login_info.username,
        &login_info.password,
    )
    .await
    .map_err(APIError::InternalError)?;

    let Some(logged_in_user) = login_result_details else {
        return Ok(
            HttpResponse::Forbidden().json(ErrorReasonResponse::custom_reason(
                "Invalid login credentials.",
            )),
        );
    };


    // Generate access and refresh token.
    let access_token_claims = JWTClaims::create(
        logged_in_user.id,
        Utc::now(),
        Duration::days(1),
        JWTTokenType::Access,
    );

    let refresh_token_claims = JWTClaims::create(
        logged_in_user.id,
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




/// Information with which to refresh a user's login, generating a new access token.
#[derive(Deserialize, ToSchema)]
#[schema(
    example = json!({
        "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJTdGFyaSBLb2xvbW9uaSIsInN\
                          1YiI6IkFQSSB0b2tlbiIsImlhdCI6MTY4Nzk3MTMyMiwiZXhwIjoxNjg4NTc2MTIyLCJ1c2V\
                          ybmFtZSI6InRlc3QiLCJ0b2tlbl90eXBlIjoicmVmcmVzaCJ9.Ze6DI5EZ-swXRQrMW3NIpp\
                          YejclGbyI9D6zmYBWJMLk"
    })
)]
pub struct UserLoginRefreshRequest {
    /// Refresh token to use to generate an access token.
    ///
    /// Token must not have expired to work.
    pub refresh_token: String,
}

/// Response on successful login refresh.
#[derive(Serialize, Debug, ToSchema)]
#[schema(
    example = json!({
        "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJTdGFyaSBLb2xvbW9uaSIsInN1\
                         YiI6IkFQSSB0b2tlbiIsImlhdCI6MTY4Nzk3MTMyMiwiZXhwIjoxNjg4MDU3NzIyLCJ1c2Vyb\
                         mFtZSI6InRlc3QiLCJ0b2tlbl90eXBlIjoiYWNjZXNzIn0.ZnuhEVacQD_pYzkW9h6aX3eoRN\
                         OAs2-y3EngGBglxkk"
    })
)]
pub struct UserLoginRefreshResponse {
    /// Newly-generated access token to use in future requests.
    pub access_token: String,
}

impl_json_response_builder!(UserLoginRefreshResponse);



/// Refresh a login
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
        content = UserLoginRefreshRequest,
    ),
    responses(
        (
            status = 200,
            description = "Login refresh successful.",
            body = UserLoginRefreshResponse
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
        openapi::MissingOrInvalidJsonRequestBodyResponse,
        openapi::InternalServerErrorResponse,
    )
)]
#[post("/refresh")]
pub async fn refresh_login(
    state: ApplicationState,
    refresh_info: web::Json<UserLoginRefreshRequest>,
) -> EndpointResult {
    // Parse and validate provided refresh token.
    let refresh_token_claims = match state.jwt_manager.decode_token(&refresh_info.refresh_token) {
        Ok(token_claims) => token_claims,
        Err(error) => {
            return match error {
                JWTValidationError::Expired(token_claims) => {
                    debug!(
                        user_id = token_claims.user_id,
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
        refresh_token_claims.user_id,
        Utc::now(),
        Duration::days(1),
        JWTTokenType::Access,
    );
    let access_token = state
        .jwt_manager
        .create_token(access_token_claims)
        .map_err(APIError::InternalError)?;


    debug!(
        user_id = refresh_token_claims.user_id,
        "User has successfully refreshed access token."
    );


    Ok(UserLoginRefreshResponse { access_token }.into_response())
}



#[rustfmt::skip]
pub fn login_router() -> Scope {
    web::scope("/login")
        .service(login)
        .service(refresh_login)
}
