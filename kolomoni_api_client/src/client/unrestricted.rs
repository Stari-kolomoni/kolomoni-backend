use std::{future::Future, pin::Pin};

use crate::{
    api::{
        auth::AuthenticationApi,
        dictionary::{
            categories::DictionaryCategoriesAuthenticatedApi,
            english::EnglishDictionaryAuthenticatedApi,
            slovene::SloveneDictionaryAuthenticatedApi,
            translation::TranslationsAuthenticatedApi,
        },
        health::HealthApi,
        users::{current::CurrentUserApi, SpecificUserAuthenticatedApi},
    },
    client::{errors::RequestResult, KolomoniHttpClient, UnauthenticatedKolomoniClient},
    request::raw::RawRequest,
    response::RawResponse,
};

#[derive(Debug, Clone)]
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
    #[inline(always)]
    pub fn authentication<'c>(&'c self) -> AuthenticationApi<'c, Self> {
        AuthenticationApi::new(self)
    }

    #[inline(always)]
    pub fn users<'c>(&'c self) -> SpecificUserAuthenticatedApi<'c, Self> {
        SpecificUserAuthenticatedApi::new(self)
    }

    #[inline(always)]
    pub fn health<'c>(&'c self) -> HealthApi<'c, Self> {
        HealthApi::new(self)
    }

    #[inline(always)]
    pub fn categories<'c>(&'c self) -> DictionaryCategoriesAuthenticatedApi<'c, Self> {
        DictionaryCategoriesAuthenticatedApi::new(self)
    }

    #[inline(always)]
    pub fn english_dictionary<'c>(&'c self) -> EnglishDictionaryAuthenticatedApi<'c, Self> {
        EnglishDictionaryAuthenticatedApi::new(self)
    }

    #[inline(always)]
    pub fn slovene_dictionary<'c>(&'c self) -> SloveneDictionaryAuthenticatedApi<'c, Self> {
        SloveneDictionaryAuthenticatedApi::new(self)
    }

    #[inline(always)]
    pub fn translations<'c>(&'c self) -> TranslationsAuthenticatedApi<'c, Self> {
        TranslationsAuthenticatedApi::new(self)
    }

    #[inline(always)]
    pub fn current_user<'c>(&'c self) -> CurrentUserApi<'c, Self> {
        CurrentUserApi::new(self)
    }
}

impl KolomoniHttpClient for UnrestrictedUnauthenticatedKolomoniClient {
    fn http_client(&self) -> &reqwest::Client {
        self.client.http_client()
    }

    fn api_server(&self) -> &crate::server::KolomoniApiServer {
        self.client.api_server()
    }

    fn send<'a>(
        &'a self,
        request: RawRequest,
    ) -> Pin<Box<dyn Future<Output = RequestResult<RawResponse>> + Send + 'a>> {
        self.client.send(request)
    }
}
