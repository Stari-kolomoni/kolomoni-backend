use std::rc::Rc;

use reqwest::Body;
use url::Url;

use crate::{
    authentication::AccessToken,
    errors::{ClientError, ClientInitializationError, ClientResult},
    response::ServerResponse,
    ApiServer,
};

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

    pub fn with_authentication(&self, authentication: &Rc<AccessToken>) -> AuthenticatedClient {
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
