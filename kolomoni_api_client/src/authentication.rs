use std::sync::Arc;

use kolomoni_core::api_models::{UserLoginRequest, UserLoginResponse};
use reqwest::StatusCode;
use thiserror::Error;

use crate::{errors::ClientError, request::RequestBuilder, UnauthenticatedKolomoniClient};


#[derive(Debug, Error)]
pub enum AuthenticationError {
    #[error("the provided login information is invalid")]
    IncorrectLoginInformation,

    #[error(transparent)]
    ClientError(#[from] ClientError),
}


#[derive(Clone)]
pub struct ServerAuthentication {
    tokens: Arc<ServerTokenSet>,
}

impl ServerAuthentication {
    pub fn new_from_token_set(access_token: ServerTokenSet) -> Self {
        Self {
            tokens: Arc::new(access_token),
        }
    }

    pub async fn new_by_server_log_in<U, P>(
        client: &UnauthenticatedKolomoniClient,
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
            .send_unauthenticated()
            .await?;

        if login_response.status() == StatusCode::OK {
            let login_response_data = login_response.json::<UserLoginResponse>().await?;

            let access_token = ServerTokenSet::new(
                login_response_data.access_token,
                login_response_data.refresh_token,
            );

            Ok(Self::new_from_token_set(access_token))
        } else if login_response.status() == StatusCode::FORBIDDEN {
            Err(AuthenticationError::IncorrectLoginInformation)
        } else {
            Err(ClientError::unexpected_status_code(login_response.status()).into())
        }
    }

    pub fn access_token(&self) -> &str {
        self.tokens.access_token()
    }
}


pub struct ServerTokenSet {
    access_token: String,

    #[allow(dead_code)]
    refresh_token: String,
}

impl ServerTokenSet {
    pub fn new(access_token: String, refresh_token: String) -> Self {
        Self {
            access_token,
            refresh_token,
        }
    }

    pub(crate) fn access_token(&self) -> &str {
        &self.access_token
    }
}
