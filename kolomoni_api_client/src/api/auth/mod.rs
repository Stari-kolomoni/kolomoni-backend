use kolomoni_core::api_models::{
    LoginErrorReason,
    UserInfo,
    UserLoginRefreshRequest,
    UserLoginRefreshResponse,
    UserLoginRequest,
    UserLoginResponse,
    UserRegistrationRequest,
    UserRegistrationResponse,
    UsersErrorReason,
};
use reqwest::StatusCode;
use thiserror::Error;

use crate::errors::{ClientError, ClientResult};
use crate::macros::{handle_uncaught_status_code, handle_unexpected_error_reason};
use crate::request::ApiClientRequestBuild;
use crate::{Client, UnauthenticatedHttpClient};


#[derive(Debug)]
pub struct UserRegistrationInfo {
    pub username: String,
    pub display_name: String,
    pub password: String,
}


#[derive(Debug)]
pub struct NewUserInfo {
    pub user: UserInfo,
}


#[derive(Debug, Error)]
pub enum UserRegistrationError {
    #[error("the provided username is already in use")]
    UsernameAlreadyExists,

    #[error("the provided display name is already in use")]
    DisplayNameAlreadyExists,

    #[error(transparent)]
    ClientError {
        #[from]
        error: ClientError,
    },
}


async fn register_user<C>(
    client: &C,
    user_registration_info: UserRegistrationInfo,
) -> ClientResult<NewUserInfo, UserRegistrationError>
where
    C: UnauthenticatedHttpClient,
{
    let response = client
        .post_request_builder()
        .endpoint_url("/users")
        .json(&UserRegistrationRequest {
            username: user_registration_info.username,
            display_name: user_registration_info.display_name,
            password: user_registration_info.password,
        })
        .send_unauthenticated()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        let response_data = response.json::<UserRegistrationResponse>().await?;

        Ok(NewUserInfo {
            user: response_data.user,
        })
    } else if response_status == StatusCode::CONFLICT {
        let users_error_reason = response.users_error_reason().await?;

        match users_error_reason {
            UsersErrorReason::UsernameAlreadyExists => {
                Err(UserRegistrationError::UsernameAlreadyExists)
            }
            UsersErrorReason::DisplayNameAlreadyExists => {
                Err(UserRegistrationError::DisplayNameAlreadyExists)
            }
            _ => handle_unexpected_error_reason!(users_error_reason, response_status),
        }
    } else {
        handle_uncaught_status_code!(response_status);
    }
}



pub struct UserLoginInfo {
    pub username: String,
    pub password: String,
}

pub struct AccessAndRefreshToken {
    pub access_token: String,
    pub refresh_token: String,
}


#[derive(Debug, Error)]
pub enum UserLoginError {
    #[error("the provided login credentials are not valid")]
    IncorrectCredentials,

    #[error(transparent)]
    ClientError {
        #[from]
        error: ClientError,
    },
}


async fn login_user<C>(
    client: &C,
    user_login_credentials: UserLoginInfo,
) -> ClientResult<AccessAndRefreshToken, UserLoginError>
where
    C: UnauthenticatedHttpClient,
{
    let response = client
        .post_request_builder()
        .endpoint_url("/auth/login")
        .json(&UserLoginRequest {
            username: user_login_credentials.username,
            password: user_login_credentials.password,
        })
        .send_unauthenticated()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        let login_response = response.json::<UserLoginResponse>().await?;

        Ok(AccessAndRefreshToken {
            access_token: login_response.access_token,
            refresh_token: login_response.refresh_token,
        })
    } else if response_status == StatusCode::FORBIDDEN {
        let login_error_reason = response.login_error_reason().await?;

        match login_error_reason {
            LoginErrorReason::InvalidLoginCredentials => Err(UserLoginError::IncorrectCredentials),
            _ => handle_unexpected_error_reason!(login_error_reason, response_status),
        }
    } else {
        handle_uncaught_status_code!(response_status);
    }
}


pub struct UserLoginRefreshInfo {
    pub refresh_token: String,
}


pub struct RefreshedAccessToken {
    pub access_token: String,
}


#[derive(Debug, Error)]
pub enum UserLoginRefreshError {
    #[error("the provided refresh token has expired")]
    RefreshTokenHasExpired,

    #[error("the provided refresh token is invalid (not a valid JWT)")]
    RefreshTokenIsInvalid,

    #[error("the provided token is a JWT, but is not a refresh token")]
    TokenIsNotARefreshToken,

    #[error(transparent)]
    ClientError {
        #[from]
        error: ClientError,
    },
}


async fn refresh_user_login<C>(
    client: &C,
    refresh_token_info: UserLoginRefreshInfo,
) -> ClientResult<RefreshedAccessToken, UserLoginRefreshError>
where
    C: UnauthenticatedHttpClient,
{
    let response = client
        .post_request_builder()
        .endpoint_url("/auth/login/refresh")
        .json(&UserLoginRefreshRequest {
            refresh_token: refresh_token_info.refresh_token,
        })
        .send_unauthenticated()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        let newly_refreshed_token_info = response.json::<UserLoginRefreshResponse>().await?;

        Ok(RefreshedAccessToken {
            access_token: newly_refreshed_token_info.access_token,
        })
    } else if response_status == StatusCode::BAD_REQUEST {
        let login_error_reason = response.login_error_reason().await?;

        match login_error_reason {
            LoginErrorReason::ExpiredRefreshToken => {
                Err(UserLoginRefreshError::RefreshTokenHasExpired)
            }
            LoginErrorReason::InvalidRefreshJsonWebToken => {
                Err(UserLoginRefreshError::RefreshTokenIsInvalid)
            }
            LoginErrorReason::NotARefreshToken => {
                Err(UserLoginRefreshError::TokenIsNotARefreshToken)
            }
            _ => handle_unexpected_error_reason!(login_error_reason, response_status),
        }
    } else {
        handle_uncaught_status_code!(response_status);
    }
}



pub struct AuthenticationApi<'c, C>
where
    C: Client,
{
    client: &'c C,
}

impl<'c, C> AuthenticationApi<'c, C>
where
    C: Client + UnauthenticatedHttpClient,
{
    pub(crate) const fn new(client: &'c C) -> Self {
        Self { client }
    }

    pub async fn register_user(
        &self,
        user_registration_info: UserRegistrationInfo,
    ) -> ClientResult<NewUserInfo, UserRegistrationError> {
        register_user(self.client, user_registration_info).await
    }

    pub async fn login_user(
        &self,
        user_login_credentials: UserLoginInfo,
    ) -> ClientResult<AccessAndRefreshToken, UserLoginError> {
        login_user(self.client, user_login_credentials).await
    }

    pub async fn refresh_user_login(
        &self,
        refresh_token_info: UserLoginRefreshInfo,
    ) -> ClientResult<RefreshedAccessToken, UserLoginRefreshError> {
        refresh_user_login(self.client, refresh_token_info).await
    }
}
