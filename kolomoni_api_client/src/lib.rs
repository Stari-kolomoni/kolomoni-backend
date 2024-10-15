//! A client for Stari Kolomoni implementing **(!) a subset of its API (!)**.

use std::rc::Rc;

use authentication::AccessToken;
use errors::{ClientError, ClientResult};
use kolomoni_core::api_models::{CategoryErrorReason, ErrorReason, ResponseWithErrorReason};
use reqwest::{header::HeaderMap, Body, StatusCode, Url};
use serde::de::DeserializeOwned;
use server::ApiServer;
use thiserror::Error;

pub(crate) mod macros;

pub mod api;
pub mod authentication;
pub mod errors;
pub(crate) mod request;
pub mod server;


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

    #[deprecated]
    pub(crate) async fn prefetch_body(self) -> ClientResult<PrefetchedServerResponse> {
        let response_status_code = self.http_response.status();
        let response_headers = self.http_response.headers().to_owned();

        let response_body_bytes = self
            .http_response
            .bytes()
            .await
            .map_err(|error| ClientError::RequestExecutionError { error })?;


        Ok(PrefetchedServerResponse {
            response_status_code,
            response_headers,
            response_body_bytes,
            cached_error_reason: None,
        })
    }

    pub(crate) fn status(&self) -> StatusCode {
        self.http_response.status()
    }

    pub(crate) async fn json<V>(self) -> ClientResult<V>
    where
        V: DeserializeOwned,
    {
        let body_data = self
            .http_response
            .bytes()
            .await
            .map_err(|error| ClientError::RequestExecutionError { error })?;

        serde_json::from_slice(&body_data)
            .map_err(|error| ClientError::ResponseJsonBodyError { error })
    }

    pub(crate) async fn json_error_reason(self) -> ClientResult<ErrorReason> {
        let response_with_error_reason = self.json::<ResponseWithErrorReason>().await?;

        Ok(response_with_error_reason.reason)
    }

    pub(crate) async fn json_category_error_reason(self) -> ClientResult<CategoryErrorReason> {
        let response_status = self.status();
        let error_reason = self.json_error_reason().await?;

        let ErrorReason::Category(category_error_reason) = error_reason else {
            return Err(ClientError::unexpected_error_reason(
                error_reason,
                response_status,
            ));
        };

        Ok(category_error_reason)
    }
}


#[deprecated]
pub struct PrefetchedServerResponse {
    response_status_code: StatusCode,
    response_headers: HeaderMap,
    response_body_bytes: bytes::Bytes,
    cached_error_reason: Option<ErrorReason>,
}

impl PrefetchedServerResponse {
    pub(crate) fn json<V>(&self) -> ClientResult<V>
    where
        V: DeserializeOwned,
    {
        serde_json::from_slice(&self.response_body_bytes)
            .map_err(|error| ClientError::ResponseJsonBodyError { error })
    }

    pub(crate) fn json_error_reason(&mut self) -> ClientResult<ErrorReason> {
        if let Some(cached_error_reason) = &self.cached_error_reason {
            return Ok(cached_error_reason.to_owned());
        }


        let response_with_error_reason = self.json::<ResponseWithErrorReason>()?;
        self.cached_error_reason = Some(response_with_error_reason.reason.clone());

        Ok(response_with_error_reason.reason)
    }

    pub(crate) fn json_category_error_reason(&mut self) -> ClientResult<CategoryErrorReason> {
        let error_reason = self.json_error_reason()?;

        let ErrorReason::Category(category_error_reason) = error_reason else {
            return Err(ClientError::unexpected_error_reason(
                error_reason,
                self.response_status_code,
            ));
        };

        Ok(category_error_reason)
    }
}



pub(crate) trait HttpClient {
    fn server(&self) -> &ApiServer;

    async fn get(&self, url: Url) -> ClientResult<ServerResponse>;

    async fn post<B>(&self, url: Url, json_body: Option<B>) -> ClientResult<ServerResponse>
    where
        B: Into<Body>;

    async fn patch<B>(&self, url: Url, json_body: Option<B>) -> ClientResult<ServerResponse>
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

    async fn get(&self, url: Url) -> ClientResult<ServerResponse> {
        self.http_client
            .get(url)
            .send()
            .await
            .map(ServerResponse::from_reqwest_response)
            .map_err(|error| ClientError::RequestExecutionError { error })
    }

    async fn post<B>(&self, url: Url, json_body: Option<B>) -> ClientResult<ServerResponse>
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

    async fn patch<B>(&self, url: Url, json_body: Option<B>) -> ClientResult<ServerResponse>
    where
        B: Into<Body>,
    {
        let mut request_builder = self.http_client.patch(url);

        if let Some(json_body) = json_body {
            request_builder = request_builder.body(json_body);
        }

        request_builder
            .send()
            .await
            .map(ServerResponse::from_reqwest_response)
            .map_err(|error| ClientError::RequestExecutionError { error })
    }
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


impl HttpClient for AuthenticatedClient {
    fn server(&self) -> &ApiServer {
        &self.server
    }

    async fn get(&self, url: Url) -> ClientResult<ServerResponse> {
        self.http_client
            .get(url)
            .bearer_auth(self.authentication.access_token())
            .send()
            .await
            .map(ServerResponse::from_reqwest_response)
            .map_err(|error| ClientError::RequestExecutionError { error })
    }

    async fn post<B>(&self, url: Url, json_body: Option<B>) -> ClientResult<ServerResponse>
    where
        B: Into<Body>,
    {
        let mut request_builder = self
            .http_client
            .post(url)
            .bearer_auth(self.authentication.access_token());

        if let Some(json_body) = json_body {
            request_builder = request_builder.body(json_body);
        }

        request_builder
            .send()
            .await
            .map(ServerResponse::from_reqwest_response)
            .map_err(|error| ClientError::RequestExecutionError { error })
    }

    async fn patch<B>(&self, url: Url, json_body: Option<B>) -> ClientResult<ServerResponse>
    where
        B: Into<Body>,
    {
        let mut request_builder = self
            .http_client
            .patch(url)
            .bearer_auth(self.authentication.access_token());

        if let Some(json_body) = json_body {
            request_builder = request_builder.body(json_body);
        }

        request_builder
            .send()
            .await
            .map(ServerResponse::from_reqwest_response)
            .map_err(|error| ClientError::RequestExecutionError { error })
    }
}
