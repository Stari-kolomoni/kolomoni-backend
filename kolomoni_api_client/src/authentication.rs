use kolomoni_core::api_models::{UserLoginRequest, UserLoginResponse};
use reqwest::StatusCode;
use thiserror::Error;

use crate::{server::ApiServer, urls::build_request_url};


#[derive(Debug, Error)]
pub enum AuthenticationError {
    #[error("failed to parse a URL")]
    UrlParseError {
        #[from]
        #[source]
        error: url::ParseError,
    },

    #[error("failed to build or perform request")]
    RequestError { error: reqwest::Error },

    #[error("failed to parse response (could be a JSON deserialization error)")]
    ResponseError { error: reqwest::Error },

    #[error("the provided login information is invalid")]
    IncorrectLoginInformation,

    #[error("unexpected status code in response: {}", .status_code)]
    UnexpectedStatusResponse { status_code: StatusCode },
}


pub struct Authentication {
    access_token: String,
}

impl Authentication {
    pub async fn log_in<U, P>(
        server: ApiServer,
        http_client: reqwest::Client,
        username: U,
        password: P,
    ) -> Result<Self, AuthenticationError>
    where
        U: Into<String>,
        P: Into<String>,
    {
        let login_response = http_client
            .post(build_request_url(&server, "/login")?)
            .json(&UserLoginRequest {
                username: username.into(),
                password: password.into(),
            })
            .send()
            .await
            .map_err(|error| AuthenticationError::RequestError { error })?;

        if login_response.status() == StatusCode::OK {
            let login_response_data: UserLoginResponse = login_response
                .json()
                .await
                .map_err(|error| AuthenticationError::ResponseError { error })?;


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
