//! A client for Stari Kolomoni implementing **a subset of its API (!)**.

use std::rc::Rc;

use authentication::Authentication;
use reqwest::Url;
use server::ApiServer;
use thiserror::Error;

pub mod api;
pub mod authentication;
pub mod server;
pub(crate) mod urls;


#[derive(Debug, Error)]
pub enum ClientError {
    #[error("failed to parse a URL")]
    UrlParseError {
        #[from]
        #[source]
        error: url::ParseError,
    },

    #[error("failed to execute request")]
    RequestExecutionError {
        #[source]
        error: reqwest::Error,
    },

    #[error("failed to extract JSON body from response")]
    ResponseJsonBodyError {
        #[source]
        error: reqwest::Error,
    },
}

pub type ClientResult<V> = Result<V, ClientError>;


#[derive(Debug, Error)]
pub enum ClientInitializationError {
    #[error("unable to initialize reqwest HTTP client")]
    UnableToInitializeReqwestClient {
        #[from]
        #[source]
        error: reqwest::Error,
    },
}




fn build_client_user_agent() -> String {
    format!(
        "kolomoni_api_client / v{}",
        env!("CARGO_PKG_VERSION")
    )
}

pub struct Client {
    server: Rc<ApiServer>,
    http_client: reqwest::Client,
}

impl Client {
    pub fn new(server: Rc<ApiServer>) -> Result<Self, ClientInitializationError> {
        let http_client = reqwest::Client::builder()
            .zstd(true)
            .user_agent(build_client_user_agent())
            .build()
            .map_err(|error| ClientInitializationError::UnableToInitializeReqwestClient { error })?;

        Ok(Self {
            server,
            http_client,
        })
    }

    pub async fn with_authentication(
        &self,
        authentication: &Rc<Authentication>,
    ) -> AuthenticatedClient {
        AuthenticatedClient::new(
            self.server.clone(),
            authentication.clone(),
            self.http_client.clone(),
        )
    }


    pub(crate) async fn get(&self, url: Url) -> Result<reqwest::Response, ClientError> {
        self.http_client
            .get(url)
            .send()
            .await
            .map_err(|error| ClientError::RequestExecutionError { error })
    }

    // TODO implement endpoints (wait, should we split into authless and authfull sections that require authentication to enter?)
}


pub struct AuthenticatedClient {
    server: Rc<ApiServer>,
    authentication: Rc<Authentication>,
    http_client: reqwest::Client,
}

impl AuthenticatedClient {
    pub(crate) fn new(
        server: Rc<ApiServer>,
        authentication: Rc<Authentication>,
        http_client: reqwest::Client,
    ) -> Self {
        Self {
            server,
            authentication,
            http_client,
        }
    }
}
