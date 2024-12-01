use std::rc::Rc;

use reqwest::Body;
use url::Url;

use crate::{
    api::{
        dictionary::{
            categories::{DictionaryCategoriesApi, DictionaryCategoriesAuthenticatedApi},
            english::{EnglishDictionaryApi, EnglishDictionaryAuthenticatedApi},
            slovene::{SloveneDictionaryApi, SloveneDictionaryAuthenticatedApi},
        },
        health::{HealthApi, HealthAuthenticatedApi},
    },
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

    async fn delete(&self, url: Url) -> ClientResult<ServerResponse>;
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
    pub fn new(server: &Rc<ApiServer>) -> Result<Self, ClientInitializationError> {
        let http_client_partial = reqwest::Client::builder()
            .zstd(true)
            .user_agent(build_client_user_agent());

        let http_client_partial = match server.is_https() {
            true => http_client_partial.https_only(true),
            false => http_client_partial,
        };

        let http_client = http_client_partial
            .build()
            .map_err(|error| ClientInitializationError::UnableToInitializeReqwestClient { error })?;

        Ok(Self {
            server: server.clone(),
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

impl Client {
    pub fn health(&self) -> HealthApi<'_> {
        HealthApi::new(self)
    }

    pub fn categories(&self) -> DictionaryCategoriesApi<'_> {
        DictionaryCategoriesApi::new(self)
    }

    pub fn english_dictionary(&self) -> EnglishDictionaryApi<'_> {
        EnglishDictionaryApi::new(self)
    }

    pub fn slovene_dictionary(&self) -> SloveneDictionaryApi<'_> {
        SloveneDictionaryApi::new(self)
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

    async fn delete(&self, url: Url) -> ClientResult<ServerResponse> {
        self.http_client
            .delete(url)
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

impl AuthenticatedClient {
    pub fn health(&self) -> HealthAuthenticatedApi<'_> {
        HealthAuthenticatedApi::new(self)
    }

    pub fn categories(&self) -> DictionaryCategoriesAuthenticatedApi<'_> {
        DictionaryCategoriesAuthenticatedApi::new(self)
    }

    pub fn english_dictionary(&self) -> EnglishDictionaryAuthenticatedApi<'_> {
        EnglishDictionaryAuthenticatedApi::new(self)
    }

    pub fn slovene_dictionary(&self) -> SloveneDictionaryAuthenticatedApi<'_> {
        SloveneDictionaryAuthenticatedApi::new(self)
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

    async fn delete(&self, url: Url) -> ClientResult<ServerResponse> {
        self.http_client
            .delete(url)
            .bearer_auth(self.authentication.access_token())
            .send()
            .await
            .map(ServerResponse::from_reqwest_response)
            .map_err(|error| ClientError::RequestExecutionError { error })
    }
}
