use actix_web::body::BoxBody;
use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use anyhow::Context;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::api::errors::{APIError, EndpointResult, ErrorReasonResponse};
use crate::api::macros::DumbResponder;
use crate::database::query;
use crate::impl_json_responder;
use crate::jwt::{JWTClaims, JWTTokenType, JWTValidationError};
use crate::state::AppState;

/*
 * POST /login
 */

#[derive(Deserialize)]
pub struct UserLoginInfo {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Debug)]
pub struct UserLoginResponse {
    pub access_token: String,
    pub refresh_token: String,
}

impl_json_responder!(UserLoginResponse);


#[post("/login")]
pub async fn login(
    state: web::Data<AppState>,
    login_info: web::Json<UserLoginInfo>,
) -> EndpointResult {
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
        .with_context(|| "Errored while creating JWT access token.")
        .map_err(APIError::InternalError)?;

    let refresh_token = state
        .jwt_manager
        .create_token(refresh_token_claims)
        .with_context(|| "Errored while creating JWT refresh token.")
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

#[derive(Deserialize)]
pub struct UserLoginRefreshInfo {
    pub refresh_token: String,
}

#[derive(Serialize, Debug)]
pub struct UserLoginRefreshResponse {
    pub access_token: String,
}

impl_json_responder!(UserLoginRefreshResponse);



#[post("/login/refresh")]
pub async fn refresh_login(
    state: web::Data<AppState>,
    refresh_info: web::Json<UserLoginRefreshInfo>,
) -> EndpointResult {
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
                            "Invalid token, could not parse.",
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
