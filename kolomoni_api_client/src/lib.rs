//! A client for Stari Kolomoni implementing **a subset of its API (!)**.

use std::rc::Rc;

use authentication::AccessToken;
use reqwest::{Body, StatusCode, Url};
use serde::de::DeserializeOwned;
use server::ApiServer;
use thiserror::Error;

pub mod api;
pub mod authentication;
pub(crate) mod request;
pub mod server;


#[derive(Debug, Error)]
pub enum ClientError {
    #[error("failed to parse a URL")]
    UrlParseError {
        #[from]
        #[source]
        error: url::ParseError,
    },

    #[error("failed to serialize data for body")]
    RequestBodySerializationError {
        #[source]
        error: serde_json::Error,
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


pub struct ServerResponse {
    http_response: reqwest::Response,
}

impl ServerResponse {
    pub(crate) fn from_reqwest_response(response: reqwest::Response) -> Self {
        Self {
            http_response: response,
        }
    }

    pub(crate) fn into_reqwest_response(self) -> reqwest::Response {
        self.http_response
    }

    pub(crate) fn status(&self) -> StatusCode {
        self.http_response.status()
    }

    pub(crate) async fn json<V>(self) -> ClientResult<V>
    where
        V: DeserializeOwned,
    {
        self.http_response
            .json()
            .await
            .map_err(|error| ClientError::ResponseJsonBodyError { error })
    }
}



pub(crate) trait HttpClient {
    fn server(&self) -> &ApiServer;

    async fn get(&self, url: Url) -> Result<ServerResponse, ClientError>;

    async fn post<B>(&self, url: Url, json_body: Option<B>) -> Result<ServerResponse, ClientError>
    where
        B: Into<Body>;
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
        authentication: &Rc<AccessToken>,
    ) -> AuthenticatedClient {
        AuthenticatedClient::new(
            self.server.clone(),
            authentication.clone(),
            self.http_client.clone(),
        )
    }
}

impl HttpClient for Client {
    fn server(&self) -> &ApiServer {
        &self.server
    }

    async fn get(&self, url: Url) -> Result<ServerResponse, ClientError> {
        self.http_client
            .get(url)
            .send()
            .await
            .map(ServerResponse::from_reqwest_response)
            .map_err(|error| ClientError::RequestExecutionError { error })
    }

    async fn post<B>(&self, url: Url, json_body: Option<B>) -> Result<ServerResponse, ClientError>
    where
        B: Into<Body>,
    {
        let mut request_builder = self.http_client.post(url);

        if let Some(json_body) = json_body {
            request_builder = request_builder.body(json_body);
        }

        request_builder
            .send()
            .await
            .map(ServerResponse::from_reqwest_response)
            .map_err(|error| ClientError::RequestExecutionError { error })
    }

    // TODO implement endpoints (wait, should we split into authless and authfull sections that require authentication to enter?)
}


pub struct AuthenticatedClient {
    server: Rc<ApiServer>,
    authentication: Rc<AccessToken>,
    http_client: reqwest::Client,
}

impl AuthenticatedClient {
    pub(crate) fn new(
        server: Rc<ApiServer>,
        authentication: Rc<AccessToken>,
        http_client: reqwest::Client,
    ) -> Self {
        Self {
            server,
            authentication,
            http_client,
        }
    }
}
