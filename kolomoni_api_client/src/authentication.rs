use kolomoni_core::api_models::{UserLoginRequest, UserLoginResponse};
use reqwest::StatusCode;
use thiserror::Error;

use crate::{request::RequestBuilder, Client, ClientError};


#[derive(Debug, Error)]
pub enum AuthenticationError {
    #[error("HTTP client error")]
    ClientError(
        #[from]
        #[source]
        ClientError,
    ),

    #[error("the provided login information is invalid")]
    IncorrectLoginInformation,

    #[error("unexpected status code in response: {}", .status_code)]
    UnexpectedStatusResponse { status_code: StatusCode },
}


pub struct AccessToken {
    access_token: String,
}

impl AccessToken {
    pub async fn log_in<U, P>(
        client: &Client,
        username: U,
        password: P,
    ) -> Result<Self, AuthenticationError>
    where
        U: Into<String>,
        P: Into<String>,
    {
        let login_response = RequestBuilder::post(client)
            .endpoint_url("/login")
            .json(&UserLoginRequest {
                username: username.into(),
                password: password.into(),
            })
            .send()
            .await?;

        if login_response.status() == StatusCode::OK {
            let login_response_data = login_response.json::<UserLoginResponse>().await?;

            Ok(Self {
                access_token: login_response_data.access_token,
            })
        } else if login_response.status() == StatusCode::FORBIDDEN {
            Err(AuthenticationError::IncorrectLoginInformation)
        } else {
            Err(AuthenticationError::UnexpectedStatusResponse {
                status_code: login_response.status(),
            })
        }
    }

    pub(crate) fn access_token(&self) -> &str {
        &self.access_token
    }
}
