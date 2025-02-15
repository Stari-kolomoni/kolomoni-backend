use std::{borrow::Cow, future::Future, sync::Arc};

use reqwest::{header::HeaderMap, Body};
use url::Url;

use crate::{
    api::{
        auth::AuthenticationApi,
        dictionary::{
            categories::{
                DictionaryCategoriesAuthenticatedApi,
                DictionaryCategoriesUnauthenticatedApi,
            },
            english::{EnglishDictionaryAuthenticatedApi, EnglishDictionaryUnauthenticatedApi},
            slovene::{SloveneDictionaryAuthenticatedApi, SloveneDictionaryUnauthenticatedApi},
            translation::TranslationsAuthenticatedApi,
        },
        health::{HealthAuthenticatedApi, HealthUnauthenticatedApi, SharedHealthEndpoints},
        users::{
            current::CurrentUserApi,
            AuthenticatedSpecificUserApi,
            SpecificUserUnauthenticatedApi,
        },
    },
    authentication::ServerAuthentication,
    errors::{ClientError, ClientInitializationError, ClientResult},
    response::ServerResponse,
    ApiServer,
};



pub trait Client: Send + Sync {
    fn server(&self) -> &ApiServer;
}


pub trait UnauthenticatedHttpClient: Client {
    fn get(
        &self,
        url: Url,
        additional_headers: HeaderMap,
    ) -> impl Future<Output = ClientResult<ServerResponse>> + Send;

    fn post<B>(
        &self,
        url: Url,
        additional_headers: HeaderMap,
        body: Option<B>,
    ) -> impl Future<Output = ClientResult<ServerResponse>> + Send
    where
        B: Into<Body> + Send;

    fn patch<B>(
        &self,
        url: Url,
        additional_headers: HeaderMap,
        body: Option<B>,
    ) -> impl Future<Output = ClientResult<ServerResponse>> + Send
    where
        B: Into<Body> + Send;

    fn delete<B>(
        &self,
        url: Url,
        additional_headers: HeaderMap,
        body: Option<B>,
    ) -> impl Future<Output = ClientResult<ServerResponse>> + Send
    where
        B: Into<Body> + Send;
}


pub trait AuthenticatedHttpClient: Client {
    fn get(
        &self,
        url: Url,
        additional_headers: HeaderMap,
    ) -> impl Future<Output = ClientResult<ServerResponse>> + Send;

    fn post<B>(
        &self,
        url: Url,
        additional_headers: HeaderMap,
        body: Option<B>,
    ) -> impl Future<Output = ClientResult<ServerResponse>> + Send
    where
        B: Into<Body> + Send;

    fn patch<B>(
        &self,
        url: Url,
        additional_headers: HeaderMap,
        body: Option<B>,
    ) -> impl Future<Output = ClientResult<ServerResponse>> + Send
    where
        B: Into<Body> + Send;

    fn delete<B>(
        &self,
        url: Url,
        additional_headers: HeaderMap,
        body: Option<B>,
    ) -> impl Future<Output = ClientResult<ServerResponse>> + Send
    where
        B: Into<Body> + Send;
}


/*
pub trait StrictlyUnauthenticatedHttpClient: UnauthenticatedHttpClient {}

#[rustfmt::skip]
pub trait StrictlyAuthenticatedHttpClient:
    UnauthenticatedHttpClient + AuthenticatedHttpClient
{}


#[cfg(feature = "unrestrict_authenticated_calls")]
pub trait AuthenticatedClient: UnauthenticatedHttpClient + AuthenticatedHttpClient {}

#[cfg(feature = "unrestrict_authenticated_calls")]
impl<C> AuthenticatedClient for C where C: UnauthenticatedHttpClient + AuthenticatedHttpClient {}


#[cfg(not(feature = "unrestrict_authenticated_calls"))]
pub trait AuthenticatedClient: StrictlyAuthenticatedHttpClient {}

#[cfg(not(feature = "unrestrict_authenticated_calls"))]
impl<C> AuthenticatedClient for C where C: StrictlyAuthenticatedHttpClient {}
 */


pub trait SharedApiClientEndpointGroups {
    type AuthenticationApi<'c>
    where
        Self: 'c;

    type UserApi<'c>
    where
        Self: 'c;

    type HealthApi<'c>: SharedHealthEndpoints
    where
        Self: 'c;

    type CategoriesApi<'c>
    where
        Self: 'c;

    type EnglishDictionaryApi<'c>
    where
        Self: 'c;

    type SloveneDictionaryApi<'c>
    where
        Self: 'c;

    fn authentication(&self) -> Self::AuthenticationApi<'_>;

    fn users(&self) -> Self::UserApi<'_>;

    fn health(&self) -> Self::HealthApi<'_>;

    fn categories(&self) -> Self::CategoriesApi<'_>;

    fn english_dictionary(&self) -> Self::EnglishDictionaryApi<'_>;

    fn slovene_dictionary(&self) -> Self::SloveneDictionaryApi<'_>;
}




const DEFAULT_CLIENT_USER_AGENT: &str = concat!(
    "kolomoni_api_client / v{}",
    env!("CARGO_PKG_VERSION")
);


pub struct ClientOptions {
    pub user_agent: Cow<'static, str>,
}


pub struct UnauthenticatedKolomoniClient {
    server: Arc<ApiServer>,
    http_client: reqwest::Client,
}

impl UnauthenticatedKolomoniClient {
    pub fn new(server: Arc<ApiServer>) -> Result<Self, ClientInitializationError> {
        Self::new_with_options(
            server,
            ClientOptions {
                user_agent: Cow::Borrowed(DEFAULT_CLIENT_USER_AGENT),
            },
        )
    }

    pub fn new_with_options(
        server: Arc<ApiServer>,
        options: ClientOptions,
    ) -> Result<Self, ClientInitializationError> {
        let http_client_partial = reqwest::Client::builder()
            .zstd(true)
            .user_agent(options.user_agent.as_ref());

        let http_client_partial = match server.is_https() {
            true => http_client_partial.https_only(true),
            false => http_client_partial,
        };

        let http_client = http_client_partial
            .build()
            .map_err(|error| ClientInitializationError::UnableToInitializeReqwestClient { error })?;

        Ok(Self {
            server,
            http_client,
        })
    }

    pub fn with_authentication(
        &self,
        authentication: ServerAuthentication,
    ) -> AuthenticatedKolomoniClient {
        AuthenticatedKolomoniClient::new(
            self.server.clone(),
            authentication.clone(),
            self.http_client.clone(),
        )
    }

    // TODO allow for request debugging outside of the crate
}

#[cfg(not(feature = "unrestrict_authenticated_calls"))]
impl SharedApiClientEndpointGroups for UnauthenticatedKolomoniClient {
    type AuthenticationApi<'c> = AuthenticationApi<'c, Self>;
    type UserApi<'c> = SpecificUserUnauthenticatedApi<'c, Self>;
    type HealthApi<'c> = HealthUnauthenticatedApi<'c, Self>;
    type CategoriesApi<'c> = DictionaryCategoriesUnauthenticatedApi<'c, Self>;
    type EnglishDictionaryApi<'c> = EnglishDictionaryUnauthenticatedApi<'c, Self>;
    type SloveneDictionaryApi<'c> = SloveneDictionaryUnauthenticatedApi<'c, Self>;

    #[inline]
    fn authentication(&self) -> Self::AuthenticationApi<'_> {
        AuthenticationApi::new(self)
    }

    #[inline]
    fn users(&self) -> Self::UserApi<'_> {
        SpecificUserUnauthenticatedApi::new(self)
    }

    #[inline]
    fn health(&self) -> HealthUnauthenticatedApi<'_, Self> {
        HealthUnauthenticatedApi::new(self)
    }

    #[inline]
    fn categories(&self) -> DictionaryCategoriesUnauthenticatedApi<'_, Self> {
        DictionaryCategoriesUnauthenticatedApi::new(self)
    }

    #[inline]
    fn english_dictionary(&self) -> EnglishDictionaryUnauthenticatedApi<'_, Self> {
        EnglishDictionaryUnauthenticatedApi::new(self)
    }

    #[inline]
    fn slovene_dictionary(&self) -> SloveneDictionaryUnauthenticatedApi<'_, Self> {
        SloveneDictionaryUnauthenticatedApi::new(self)
    }
}

#[cfg(feature = "unrestrict_authenticated_calls")]
impl SharedApiClientEndpointGroups for StrictlyUnauthenticatedHttpClient {
    type AuthenticationApi<'c> = AuthenticationApi<'c, Self>;
    type UserApi<'c> = AuthenticatedSpecificUserApi<'c, Self>;
    type HealthApi<'c> = HealthAuthenticatedApi<'c, Self>;
    type CategoriesApi<'c> = DictionaryCategoriesAuthenticatedApi<'c, Self>;
    type EnglishDictionaryApi<'c> = EnglishDictionaryAuthenticatedApi<'c, Self>;
    type SloveneDictionaryApi<'c> = SloveneDictionaryAuthenticatedApi<'c, Self>;

    fn authentication(&self) -> Self::AuthenticationApi<'_> {
        AuthenticationApi::new(self)
    }

    fn users(&self) -> Self::UserApi<'_> {
        AuthenticatedSpecificUserApi::new(self)
    }

    fn health(&self) -> Self::HealthApi<'_> {
        HealthAuthenticatedApi::new(self)
    }

    fn categories(&self) -> Self::CategoriesApi<'_> {
        DictionaryCategoriesAuthenticatedApi::new(self)
    }

    fn english_dictionary(&self) -> Self::EnglishDictionaryApi<'_> {
        EnglishDictionaryAuthenticatedApi::new(self)
    }

    fn slovene_dictionary(&self) -> Self::SloveneDictionaryApi<'_> {
        SloveneDictionaryAuthenticatedApi::new(self)
    }
}


impl Client for UnauthenticatedKolomoniClient {
    fn server(&self) -> &ApiServer {
        &self.server
    }
}

impl UnauthenticatedHttpClient for UnauthenticatedKolomoniClient {
    async fn get(&self, url: Url, additional_headers: HeaderMap) -> ClientResult<ServerResponse> {
        self.http_client
            .get(url)
            .headers(additional_headers)
            .send()
            .await
            .map(ServerResponse::from_reqwest_response)
            .map_err(|error| ClientError::RequestExecutionError { error })
    }

    async fn post<B>(
        &self,
        url: Url,
        additional_headers: HeaderMap,
        json_body: Option<B>,
    ) -> ClientResult<ServerResponse>
    where
        B: Into<Body> + Send,
    {
        let mut request_builder = self.http_client.post(url);

        if let Some(json_body) = json_body {
            request_builder = request_builder.body(json_body);
        }

        request_builder
            .headers(additional_headers)
            .send()
            .await
            .map(ServerResponse::from_reqwest_response)
            .map_err(|error| ClientError::RequestExecutionError { error })
    }

    async fn patch<B>(
        &self,
        url: Url,
        additional_headers: HeaderMap,
        json_body: Option<B>,
    ) -> ClientResult<ServerResponse>
    where
        B: Into<Body> + Send,
    {
        let mut request_builder = self.http_client.patch(url);

        if let Some(json_body) = json_body {
            request_builder = request_builder.body(json_body);
        }

        request_builder
            .headers(additional_headers)
            .send()
            .await
            .map(ServerResponse::from_reqwest_response)
            .map_err(|error| ClientError::RequestExecutionError { error })
    }

    async fn delete<B>(
        &self,
        url: Url,
        additional_headers: HeaderMap,
        json_body: Option<B>,
    ) -> ClientResult<ServerResponse>
    where
        B: Into<Body> + Send,
    {
        let mut request_builder = self.http_client.delete(url);

        if let Some(json_body) = json_body {
            request_builder = request_builder.body(json_body);
        }

        request_builder
            .headers(additional_headers)
            .send()
            .await
            .map(ServerResponse::from_reqwest_response)
            .map_err(|error| ClientError::RequestExecutionError { error })
    }
}

// impl StrictlyUnauthenticatedHttpClient for UnauthenticatedKolomoniClient {}




pub struct AuthenticatedKolomoniClient {
    server: Arc<ApiServer>,
    authentication: ServerAuthentication,
    http_client: reqwest::Client,
}

impl AuthenticatedKolomoniClient {
    pub(crate) fn new(
        server: Arc<ApiServer>,
        authentication: ServerAuthentication,
        http_client: reqwest::Client,
    ) -> Self {
        Self {
            server,
            authentication,
            http_client,
        }
    }

    pub fn without_authentication(&self) -> UnauthenticatedKolomoniClient {
        UnauthenticatedKolomoniClient {
            server: self.server.clone(),
            http_client: self.http_client.clone(),
        }
    }
}

impl SharedApiClientEndpointGroups for AuthenticatedKolomoniClient {
    type AuthenticationApi<'c> = AuthenticationApi<'c, Self>;
    type UserApi<'c> = AuthenticatedSpecificUserApi<'c, Self>;
    type HealthApi<'c> = HealthAuthenticatedApi<'c, Self>;
    type CategoriesApi<'c> = DictionaryCategoriesAuthenticatedApi<'c, Self>;
    type EnglishDictionaryApi<'c> = EnglishDictionaryAuthenticatedApi<'c, Self>;
    type SloveneDictionaryApi<'c> = SloveneDictionaryAuthenticatedApi<'c, Self>;

    #[inline]
    fn authentication(&self) -> Self::AuthenticationApi<'_> {
        AuthenticationApi::new(self)
    }

    #[inline]
    fn users(&self) -> Self::UserApi<'_> {
        AuthenticatedSpecificUserApi::new(self)
    }

    #[inline]
    fn health(&self) -> HealthAuthenticatedApi<'_, Self> {
        HealthAuthenticatedApi::new(self)
    }

    #[inline]
    fn categories(&self) -> DictionaryCategoriesAuthenticatedApi<'_, Self> {
        DictionaryCategoriesAuthenticatedApi::new(self)
    }

    #[inline]
    fn english_dictionary(&self) -> EnglishDictionaryAuthenticatedApi<'_, Self> {
        EnglishDictionaryAuthenticatedApi::new(self)
    }

    #[inline]
    fn slovene_dictionary(&self) -> SloveneDictionaryAuthenticatedApi<'_, Self> {
        SloveneDictionaryAuthenticatedApi::new(self)
    }
}

impl AuthenticatedKolomoniClient {
    pub fn translations(&self) -> TranslationsAuthenticatedApi<'_, Self> {
        TranslationsAuthenticatedApi::new(self)
    }

    pub fn current_user(&self) -> CurrentUserApi<'_, Self> {
        CurrentUserApi::new(self)
    }
}

impl Client for AuthenticatedKolomoniClient {
    fn server(&self) -> &ApiServer {
        &self.server
    }
}


impl UnauthenticatedHttpClient for AuthenticatedKolomoniClient {
    async fn get(&self, url: Url, additional_headers: HeaderMap) -> ClientResult<ServerResponse> {
        self.http_client
            .get(url)
            .headers(additional_headers)
            .send()
            .await
            .map(ServerResponse::from_reqwest_response)
            .map_err(|error| ClientError::RequestExecutionError { error })
    }

    async fn post<B>(
        &self,
        url: Url,
        additional_headers: HeaderMap,
        json_body: Option<B>,
    ) -> ClientResult<ServerResponse>
    where
        B: Into<Body> + Send,
    {
        let mut request_builder = self.http_client.post(url);

        if let Some(json_body) = json_body {
            request_builder = request_builder.body(json_body);
        }

        request_builder
            .headers(additional_headers)
            .send()
            .await
            .map(ServerResponse::from_reqwest_response)
            .map_err(|error| ClientError::RequestExecutionError { error })
    }

    async fn patch<B>(
        &self,
        url: Url,
        additional_headers: HeaderMap,
        json_body: Option<B>,
    ) -> ClientResult<ServerResponse>
    where
        B: Into<Body> + Send,
    {
        let mut request_builder = self.http_client.patch(url);

        if let Some(json_body) = json_body {
            request_builder = request_builder.body(json_body);
        }

        request_builder
            .headers(additional_headers)
            .send()
            .await
            .map(ServerResponse::from_reqwest_response)
            .map_err(|error| ClientError::RequestExecutionError { error })
    }

    async fn delete<B>(
        &self,
        url: Url,
        additional_headers: HeaderMap,
        json_body: Option<B>,
    ) -> ClientResult<ServerResponse>
    where
        B: Into<Body> + Send,
    {
        let mut request_builder = self.http_client.delete(url);

        if let Some(json_body) = json_body {
            request_builder = request_builder.body(json_body);
        }

        request_builder
            .headers(additional_headers)
            .send()
            .await
            .map(ServerResponse::from_reqwest_response)
            .map_err(|error| ClientError::RequestExecutionError { error })
    }
}

impl AuthenticatedHttpClient for AuthenticatedKolomoniClient {
    async fn get(&self, url: Url, additional_headers: HeaderMap) -> ClientResult<ServerResponse> {
        self.http_client
            .get(url)
            .bearer_auth(self.authentication.access_token())
            .headers(additional_headers)
            .send()
            .await
            .map(ServerResponse::from_reqwest_response)
            .map_err(|error| ClientError::RequestExecutionError { error })
    }

    async fn post<B>(
        &self,
        url: Url,
        additional_headers: HeaderMap,
        json_body: Option<B>,
    ) -> ClientResult<ServerResponse>
    where
        B: Into<Body> + Send,
    {
        let mut request_builder = self.http_client.post(url);

        if let Some(json_body) = json_body {
            request_builder = request_builder.body(json_body);
        }

        request_builder
            .bearer_auth(self.authentication.access_token())
            .headers(additional_headers)
            .send()
            .await
            .map(ServerResponse::from_reqwest_response)
            .map_err(|error| ClientError::RequestExecutionError { error })
    }

    async fn patch<B>(
        &self,
        url: Url,
        additional_headers: HeaderMap,
        json_body: Option<B>,
    ) -> ClientResult<ServerResponse>
    where
        B: Into<Body> + Send,
    {
        let mut request_builder = self.http_client.patch(url);

        if let Some(json_body) = json_body {
            request_builder = request_builder.body(json_body);
        }

        request_builder
            .bearer_auth(self.authentication.access_token())
            .headers(additional_headers)
            .send()
            .await
            .map(ServerResponse::from_reqwest_response)
            .map_err(|error| ClientError::RequestExecutionError { error })
    }

    async fn delete<B>(
        &self,
        url: Url,
        additional_headers: HeaderMap,
        json_body: Option<B>,
    ) -> ClientResult<ServerResponse>
    where
        B: Into<Body> + Send,
    {
        let mut request_builder = self.http_client.delete(url);

        if let Some(json_body) = json_body {
            request_builder = request_builder.body(json_body);
        }

        request_builder
            .bearer_auth(self.authentication.access_token())
            .headers(additional_headers)
            .send()
            .await
            .map(ServerResponse::from_reqwest_response)
            .map_err(|error| ClientError::RequestExecutionError { error })
    }
}

// impl StrictlyAuthenticatedHttpClient for AuthenticatedKolomoniClient {}
