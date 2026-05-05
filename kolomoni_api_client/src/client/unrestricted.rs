use crate::client::{KolomoniHttpClient, UnauthenticatedKolomoniClient};

pub struct UnrestrictedUnauthenticatedKolomoniClient {
    client: UnauthenticatedKolomoniClient,
}

impl UnrestrictedUnauthenticatedKolomoniClient {
    #[inline]
    pub fn new(unauthenticated_client: UnauthenticatedKolomoniClient) -> Self {
        Self {
            client: unauthenticated_client,
        }
    }
}

impl UnrestrictedUnauthenticatedKolomoniClient {
    pub fn authentication(&self) -> AuthenticationApi<'_, Self> {
        AuthenticationApi::new(self)
    }

    pub fn users(&self) -> AuthenticatedSpecificUserApi<'_, Self> {
        AuthenticatedSpecificUserApi::new(self)
    }

    pub fn health(&self) -> HealthAuthenticatedApi<'_, Self> {
        HealthAuthenticatedApi::new(self)
    }

    pub fn categories(&self) -> DictionaryCategoriesAuthenticatedApi<'_, Self> {
        DictionaryCategoriesAuthenticatedApi::new(self)
    }

    pub fn english_dictionary(&self) -> EnglishDictionaryAuthenticatedApi<'_, Self> {
        EnglishDictionaryAuthenticatedApi::new(self)
    }

    pub fn slovene_dictionary(&self) -> SloveneDictionaryAuthenticatedApi<'_, Self> {
        SloveneDictionaryAuthenticatedApi::new(self)
    }
}

impl KolomoniHttpClient for UnrestrictedUnauthenticatedKolomoniClient {
    fn http_client(&self) -> &reqwest::Client {
        self.client.http_client()
    }

    fn api_server(&self) -> &crate::server::KolomoniApiServer {
        self.client.api_server()
    }

    async fn send(
        &self,
        request: crate::request::RawRequest,
    ) -> super::errors::RequestResult<crate::response::raw::RawResponse> {
        self.client.send(request)
    }
}
