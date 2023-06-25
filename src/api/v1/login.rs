use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, warn};

use crate::database::queries::users;
use crate::impl_json_responder_on_serializable;
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

impl_json_responder_on_serializable!(UserLoginResponse, "UserLoginResponse");


#[post("/login")]
pub async fn login(
    request: HttpRequest,
    state: web::Data<AppState>,
    login_info: web::Json<UserLoginInfo>,
) -> impl Responder {
    let is_valid_login_result = users::Query::validate_user_credentials(
        &state.database,
        &state.hasher,
        &login_info.username,
        &login_info.password,
    )
    .await;

    let Ok(is_valid_login) = is_valid_login_result else {
        error!(
            error = is_valid_login_result.unwrap_err().to_string(),
            username = login_info.username,
            "Errored while validating login credentials."
        );

        return HttpResponse::InternalServerError()
            .finish();
    };

    if !is_valid_login {
        return HttpResponse::Forbidden().finish();
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

    let access_token = match state.jwt_manager.create_token(access_token_claims) {
        Ok(token) => token,
        Err(error) => {
            error!(
                error = error.to_string(),
                username = login_info.username,
                "Errored while creating JWT access token."
            );
            return HttpResponse::InternalServerError().finish();
        }
    };

    let refresh_token = match state.jwt_manager.create_token(refresh_token_claims) {
        Ok(token) => token,
        Err(error) => {
            error!(
                error = error.to_string(),
                username = login_info.username,
                "Errored while creating JWT refresh token."
            );
            return HttpResponse::InternalServerError().finish();
        }
    };

    debug!(
        username = login_info.username,
        "User has successfully logged in."
    );

    UserLoginResponse {
        access_token,
        refresh_token,
    }
    .respond_to(&request)
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

impl_json_responder_on_serializable!(
    UserLoginRefreshResponse,
    "UserLoginRefreshResponse"
);



#[post("/login/refresh")]
pub async fn refresh_login(
    request: HttpRequest,
    state: web::Data<AppState>,
    refresh_info: web::Json<UserLoginRefreshInfo>,
) -> impl Responder {
    let refresh_token_claims = match state.jwt_manager.decode_token(&refresh_info.refresh_token) {
        Ok(token_claims) => token_claims,
        Err(error) => {
            return match error {
                JWTValidationError::Expired(token_claims) => {
                    debug!(
                        username = token_claims.username,
                        "Refusing to refresh expired token.",
                    );

                    HttpResponse::Forbidden().finish()
                }
                JWTValidationError::InvalidToken(error) => {
                    warn!(error = error, "Failed to parse refresh token.",);

                    HttpResponse::BadRequest().finish()
                }
            }
        }
    };

    if refresh_token_claims.token_type != JWTTokenType::Refresh {
        return HttpResponse::BadRequest().finish();
    }

    // Refresh token is valid, create new access token.
    let access_token_claims = JWTClaims::create(
        refresh_token_claims.username.clone(),
        Utc::now(),
        Duration::days(1),
        JWTTokenType::Access,
    );
    let access_token = match state.jwt_manager.create_token(access_token_claims) {
        Ok(token) => token,
        Err(error) => {
            error!(
                error = error.to_string(),
                username = refresh_token_claims.username,
                "Errored while creating refreshed JWT access token."
            );
            return HttpResponse::InternalServerError().finish();
        }
    };

    debug!(
        username = refresh_token_claims.username,
        "User has successfully refreshed access token."
    );

    UserLoginRefreshResponse { access_token }.respond_to(&request)
}
