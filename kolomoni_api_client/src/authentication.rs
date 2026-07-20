use kolomoni_core::api_models::{UserLoginRequest, UserLoginResponse};
use reqwest::StatusCode;
use thiserror::Error;

use crate::{
    client::{errors::RequestError, KolomoniHttpClient},
    parsing::unexpected_response,
    request::{
        typed::{BoundTypedRequest, IntoBoundTypedRequest},
        ToRequestBuilder,
    },
    response::{raw::RawResponse, ResponseValueError},
};


#[derive(Debug, Error)]
pub enum AuthenticationError {
    #[error("the provided login information is invalid")]
    IncorrectLoginInformation,

    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for AuthenticationError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}


/// An authentication store (i.e. an access token and a refresh token).
///
/// To log in, use [`Self::new_by_server_log_in`], obtaining [`Self`],
/// which can be used to obtain an authenticated client
/// (see [`UnauthanticatedTestServerClient::with_authentication`]).
#[derive(Debug, Clone)]
pub struct ClientAuthentication {
    access_token: String,

    // TODO This needs auto-refreshing, this is currently unused.
    #[allow(dead_code)]
    refresh_token: String,
}

impl ClientAuthentication {
    pub fn new(access_token: String, refresh_token: String) -> Self {
        Self {
            access_token,
            refresh_token,
        }
    }

    pub async fn new_by_server_log_in<'c, C, U, P>(
        client: &'c C,
        username: U,
        password: P,
    ) -> BoundTypedRequest<'c, C, Self, AuthenticationError>
    where
        C: KolomoniHttpClient,
        U: Into<String>,
        P: Into<String>,
    {
        let response_parser = async |response: RawResponse| {
            let status = response.status();

            if status == StatusCode::OK {
                let login_response_data = response.into_json_body::<UserLoginResponse>().await?;

                Ok(Self::new(
                    login_response_data.access_token,
                    login_response_data.refresh_token,
                ))
            } else if status == StatusCode::FORBIDDEN {
                Err(AuthenticationError::IncorrectLoginInformation)
            } else {
                Err(unexpected_response(response).await)
            }
        };

        client
            .post()
            .endpoint_url("/login")
            .json(&UserLoginRequest {
                username: username.into(),
                password: password.into(),
            })
            .build_request()
            .into_bound_typed_request(response_parser)
    }

    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    pub fn refresh_token(&self) -> &str {
        &self.refresh_token
    }
}
