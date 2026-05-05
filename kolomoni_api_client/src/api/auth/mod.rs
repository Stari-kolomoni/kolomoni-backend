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

use crate::{
    api::EndpointGroup,
    client::{errors::RequestError, KolomoniHttpClient},
    parsing::{unexpected_error_reason, unexpected_response},
    request::{
        typed::{BoundTypedRequest, IntoBoundTypedRequest},
        ToRequestBuilder,
    },
    response::{raw::RawResponse, ResponseValueError},
};

#[derive(Debug)]
pub struct UserRegistration {
    pub username: String,
    pub display_name: String,
    pub password: String,
}

#[derive(Debug)]
pub struct NewUser {
    pub user: UserInfo,
}

#[derive(Debug, Error)]
pub enum UserRegistrationError {
    #[error("the provided username is already in use")]
    UsernameAlreadyExists,

    #[error("the provided display name is already in use")]
    DisplayNameAlreadyExists,

    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for UserRegistrationError {
    fn from_request_error(client_error: RequestError) -> Self {
        Self::RequestError(client_error)
    }
}


#[inline]
fn register_user_request<'c, C>(
    client: &'c C,
    user_registration_info: UserRegistration,
) -> BoundTypedRequest<'c, C, NewUser, UserRegistrationError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let registration_response = response
                .into_json_body::<UserRegistrationResponse>()
                .await?;

            Ok(NewUser {
                user: registration_response.user,
            })
        } else if status == StatusCode::CONFLICT {
            let users_error_reason = response.users_error_reason().await?;

            match users_error_reason {
                UsersErrorReason::UsernameAlreadyExists => {
                    Err(UserRegistrationError::UsernameAlreadyExists)
                }
                UsersErrorReason::DisplayNameAlreadyExists => {
                    Err(UserRegistrationError::DisplayNameAlreadyExists)
                }
                _ => Err(unexpected_error_reason(
                    status,
                    users_error_reason,
                )),
            }
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .post()
        .endpoint_url("/auth/register")
        .json(&UserRegistrationRequest {
            username: user_registration_info.username,
            display_name: user_registration_info.display_name,
            password: user_registration_info.password,
        })
        .build_request()
        .into_bound_typed_request(response_parser)
}


pub struct UserLoginCredentials {
    pub username: String,
    pub password: String,
}

pub struct UserAccessCredentials {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Error)]
pub enum UserLoginError {
    #[error("the provided login credentials are not valid")]
    IncorrectCredentials,

    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for UserLoginError {
    fn from_request_error(client_error: RequestError) -> Self {
        Self::RequestError(client_error)
    }
}

fn login_user_request<'c, C>(
    client: &'c C,
    credentials: UserLoginCredentials,
) -> BoundTypedRequest<'c, C, UserAccessCredentials, UserLoginError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let login_response = response.into_json_body::<UserLoginResponse>().await?;

            Ok(UserAccessCredentials {
                access_token: login_response.access_token,
                refresh_token: login_response.refresh_token,
            })
        } else if status == StatusCode::FORBIDDEN {
            let login_error_reason = response.login_error_reason().await?;

            match login_error_reason {
                LoginErrorReason::InvalidLoginCredentials => {
                    Err(UserLoginError::IncorrectCredentials)
                }
                _ => Err(unexpected_error_reason(
                    status,
                    login_error_reason,
                )),
            }
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .post()
        .endpoint_url("/auth/login")
        .json(&UserLoginRequest {
            username: credentials.username,
            password: credentials.password,
        })
        .build_request()
        .into_bound_typed_request(response_parser)
}

pub struct UserRefreshToken {
    pub refresh_token: String,
}

pub struct FreshAccessToken {
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
    RequestError(#[from] RequestError),
}

impl ResponseValueError for UserLoginRefreshError {
    fn from_request_error(client_error: RequestError) -> Self {
        Self::RequestError(client_error)
    }
}


#[inline]
fn refresh_login_request<'c, C>(
    client: &'c C,
    refresh_token: UserRefreshToken,
) -> BoundTypedRequest<'c, C, FreshAccessToken, UserLoginRefreshError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let fresh_token = response
                .into_json_body::<UserLoginRefreshResponse>()
                .await?;

            Ok(FreshAccessToken {
                access_token: fresh_token.access_token,
            })
        } else if status == StatusCode::BAD_REQUEST {
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
                _ => Err(unexpected_error_reason(
                    status,
                    login_error_reason,
                )),
            }
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .post()
        .endpoint_url("/auth/login/refresh")
        .json(&UserLoginRefreshRequest {
            refresh_token: refresh_token.refresh_token,
        })
        .build_request()
        .into_bound_typed_request(response_parser)
}


pub trait AuthenticationApiAnonymousEndpoints<'c, C>: EndpointGroup<'c, C>
where
    C: KolomoniHttpClient,
{
    fn register_user(
        &'c self,
        user_registration_info: UserRegistration,
    ) -> BoundTypedRequest<'c, C, NewUser, UserRegistrationError> {
        register_user_request(self.client(), user_registration_info)
    }

    fn login_user(
        &'c self,
        credentials: UserLoginCredentials,
    ) -> BoundTypedRequest<'c, C, UserAccessCredentials, UserLoginError> {
        login_user_request(self.client(), credentials)
    }

    fn refresh_user_login(
        &'c self,
        refresh_token: UserRefreshToken,
    ) -> BoundTypedRequest<'c, C, FreshAccessToken, UserLoginRefreshError> {
        refresh_login_request(self.client(), refresh_token)
    }
}


pub struct AuthenticationApi<'c, C>
where
    C: KolomoniHttpClient,
{
    client: &'c C,
}

impl<'c, C> AuthenticationApi<'c, C>
where
    C: KolomoniHttpClient,
{
    pub(crate) const fn new(client: &'c C) -> Self {
        Self { client }
    }
}

impl<'c, C> EndpointGroup<'c, C> for AuthenticationApi<'c, C>
where
    C: KolomoniHttpClient,
{
    fn client(&'c self) -> &'c C {
        self.client
    }
}

impl<'c, C> AuthenticationApiAnonymousEndpoints<'c, C> for AuthenticationApi<'c, C> where
    C: KolomoniHttpClient
{
}
