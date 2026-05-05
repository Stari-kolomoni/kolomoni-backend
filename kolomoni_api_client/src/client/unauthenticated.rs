use std::future::Future;
use std::pin::Pin;
use std::{borrow::Cow, sync::Arc};

use crate::api::auth::AuthenticationApi;
use crate::api::dictionary::categories::DictionaryCategoriesUnauthenticatedApi;
use crate::api::dictionary::english::EnglishDictionaryUnauthenticatedApi;
use crate::api::dictionary::slovene::SloveneDictionaryUnauthenticatedApi;
use crate::api::health::HealthApi;
use crate::api::users::SpecificUserUnauthenticatedApi;
use crate::client::execute_prepared_raw_request;
use crate::request::raw::RawRequest;
use crate::{
    authentication::ClientAuthentication,
    client::{
        errors::RequestResult,
        AuthenticatedKolomoniClient,
        ClientInitializationError,
        KolomoniHttpClient,
        DEFAULT_CLIENT_USER_AGENT,
    },
    response::raw::RawResponse,
    server::KolomoniApiServer,
};


pub struct UnauthenticatedClientOptions {
    pub user_agent: Cow<'static, str>,
}

impl Default for UnauthenticatedClientOptions {
    /// Constructs a default [`ClientOptions`]:
    /// - default User-Agent (containing the crate name and its version, *recommended*).
    fn default() -> Self {
        Self {
            user_agent: Cow::Borrowed(DEFAULT_CLIENT_USER_AGENT),
        }
    }
}


pub struct UnauthenticatedKolomoniClient {
    api_server: Arc<KolomoniApiServer>,
    http_client: reqwest::Client,
}

impl UnauthenticatedKolomoniClient {
    pub fn new(api_server: Arc<KolomoniApiServer>) -> Result<Self, ClientInitializationError> {
        Self::new_with_options(
            api_server,
            UnauthenticatedClientOptions::default(),
        )
    }

    pub(crate) fn new_with_http_client(
        api_server: Arc<KolomoniApiServer>,
        http_client: reqwest::Client,
    ) -> Self {
        Self {
            api_server,
            http_client,
        }
    }

    pub fn new_with_options(
        server: Arc<KolomoniApiServer>,
        options: UnauthenticatedClientOptions,
    ) -> Result<Self, ClientInitializationError> {
        let http_client_partial = reqwest::Client::builder()
            .gzip(true)
            .zstd(true)
            .user_agent(options.user_agent.as_ref());

        let http_client_partial = match server.is_https() {
            true => http_client_partial.https_only(true),
            false => http_client_partial,
        };

        let http_client = http_client_partial
            .build()
            .map_err(|error| ClientInitializationError::UnableToInitializeHttpClient { error })?;

        Ok(Self {
            api_server: server,
            http_client,
        })
    }

    pub fn into_authenticated(
        self,
        authentication: ClientAuthentication,
    ) -> AuthenticatedKolomoniClient {
        AuthenticatedKolomoniClient::new(self.api_server, authentication, self.http_client)
    }

    pub fn with_authentication(
        &self,
        authentication: ClientAuthentication,
    ) -> AuthenticatedKolomoniClient {
        AuthenticatedKolomoniClient::new(
            self.api_server.clone(),
            authentication,
            self.http_client.clone(),
        )
    }

    #[cfg(feature = "unrestricted_client")]
    pub fn into_unrestricted_client(
        self,
    ) -> super::unrestricted::UnrestrictedUnauthenticatedKolomoniClient {
        super::unrestricted::UnrestrictedUnauthenticatedKolomoniClient::new(self)
    }
}


impl KolomoniHttpClient for UnauthenticatedKolomoniClient {
    fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }

    fn api_server(&self) -> &KolomoniApiServer {
        &self.api_server
    }

    fn send<'a>(
        &'a self,
        request: RawRequest,
    ) -> Pin<Box<dyn Future<Output = RequestResult<RawResponse>> + Send + 'a>> {
        Box::pin(execute_prepared_raw_request(
            &self.http_client,
            request,
        ))
    }
}

impl UnauthenticatedKolomoniClient {
    #[inline(always)]
    pub fn authentication<'c>(&'c self) -> AuthenticationApi<'c, Self> {
        AuthenticationApi::new(self)
    }

    #[inline(always)]
    pub fn users<'c>(&'c self) -> SpecificUserUnauthenticatedApi<'c, Self> {
        SpecificUserUnauthenticatedApi::new(self)
    }

    #[inline(always)]
    pub fn health<'c>(&'c self) -> HealthApi<'c, Self> {
        HealthApi::new(self)
    }

    #[inline(always)]
    pub fn categories<'c>(&'c self) -> DictionaryCategoriesUnauthenticatedApi<'c, Self> {
        DictionaryCategoriesUnauthenticatedApi::new(self)
    }

    #[inline(always)]
    pub fn english_dictionary<'c>(&'c self) -> EnglishDictionaryUnauthenticatedApi<'c, Self> {
        EnglishDictionaryUnauthenticatedApi::new(self)
    }

    #[inline(always)]
    pub fn slovene_dictionary<'c>(&'c self) -> SloveneDictionaryUnauthenticatedApi<'c, Self> {
        SloveneDictionaryUnauthenticatedApi::new(self)
    }
}
