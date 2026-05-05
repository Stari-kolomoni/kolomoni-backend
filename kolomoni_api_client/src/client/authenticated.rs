use std::{future::Future, pin::Pin, sync::Arc};

use reqwest::header::{self, HeaderValue};

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
    authentication::ClientAuthentication,
    client::{
        errors::{RequestPreparationError, RequestResult},
        execute_prepared_raw_request,
        KolomoniHttpClient,
        UnauthenticatedKolomoniClient,
    },
    request::raw::{AuthenticationOverride, RawRequest},
    response::raw::RawResponse,
    server::KolomoniApiServer,
};

pub struct AuthenticatedKolomoniClient {
    server: Arc<KolomoniApiServer>,
    authentication: ClientAuthentication,
    http_client: reqwest::Client,
}

impl AuthenticatedKolomoniClient {
    pub(crate) fn new(
        server: Arc<KolomoniApiServer>,
        authentication: ClientAuthentication,
        http_client: reqwest::Client,
    ) -> Self {
        Self {
            server,
            authentication,
            http_client,
        }
    }

    pub fn unauthenticated(&self) -> UnauthenticatedKolomoniClient {
        UnauthenticatedKolomoniClient::new_with_http_client(
            self.server.clone(),
            self.http_client.clone(),
        )
    }

    pub fn into_unauthenticated(self) -> UnauthenticatedKolomoniClient {
        UnauthenticatedKolomoniClient::new_with_http_client(self.server, self.http_client)
    }
}


impl KolomoniHttpClient for AuthenticatedKolomoniClient {
    fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }

    fn api_server(&self) -> &KolomoniApiServer {
        &self.server
    }

    fn send<'a>(
        &'a self,
        request: RawRequest,
    ) -> Pin<Box<dyn Future<Output = RequestResult<RawResponse>> + Send + 'a>> {
        let sender_closure = async move |mut request: RawRequest| {
            // Add Authentication header if required.
            let request = match request.authentication_override() {
                AuthenticationOverride::Inherit => {
                    request.modify_request(|mut request| {
                        request.headers_mut().append(
                            header::AUTHORIZATION,
                            HeaderValue::from_str(&format!(
                                "Bearer {}",
                                self.authentication.access_token()
                            ))
                            .map_err(|error| {
                                RequestPreparationError::RequestHeaderValueEncoding {
                                    header_name: header::AUTHORIZATION,
                                    error,
                                }
                            })?,
                        );

                        Ok(request)
                    });

                    request
                }
                // If authentication was manually disabled by [`RawRequest`],
                // we do not add the authentication header.
                AuthenticationOverride::Disable => request,
            };

            execute_prepared_raw_request(&self.http_client, request).await
        };

        Box::pin(sender_closure(request))
    }
}

impl AuthenticatedKolomoniClient {
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
