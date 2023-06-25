use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::database::queries::users;
use crate::impl_json_responder_on_serializable;
use crate::jwt::JWTClaims;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct UserLoginInfo {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Debug)]
pub struct UserLoginResponse {
    pub access_token: String,
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

    let token_claims = JWTClaims::create(
        login_info.username.clone(),
        Utc::now(),
        Duration::days(7),
    );
    let token = match state.jwt_manager.create_token(token_claims) {
        Ok(token) => token,
        Err(error) => {
            error!(
                error = error.to_string(),
                username = login_info.username,
                "Errored while creating JWT token."
            );
            return HttpResponse::InternalServerError().finish();
        }
    };

    debug!(
        username = login_info.username,
        "User has successfully logged in."
    );

    UserLoginResponse {
        access_token: token,
    }
    .respond_to(&request)
}
