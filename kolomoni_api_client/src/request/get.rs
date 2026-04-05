use std::borrow::Borrow;

use reqwest::header::HeaderMap;
use url::Url;

use super::{build_request_url, build_request_url_with_parameters, UrlBuildType};
use crate::{
    errors::{ClientError, ClientResult},
    response::ServerResponse,
    AuthenticatedHttpClient,
    Client,
    UnauthenticatedHttpClient,
};



pub(crate) struct PreparedGetRequest<'c, C>
where
    C: Client,
{
    client: &'c C,

    request_url: Url,

    additional_headers: HeaderMap,
}

impl<C> PreparedGetRequest<'_, C>
where
    C: Client + UnauthenticatedHttpClient,
{
    async fn execute_unauthenticated(self) -> ClientResult<ServerResponse> {
        UnauthenticatedHttpClient::get(
            self.client,
            self.request_url,
            self.additional_headers,
        )
        .await
    }
}

impl<C> PreparedGetRequest<'_, C>
where
    C: Client + AuthenticatedHttpClient,
{
    async fn execute_authenticated(self) -> ClientResult<ServerResponse> {
        AuthenticatedHttpClient::get(
            self.client,
            self.request_url,
            self.additional_headers,
        )
        .await
    }
}



pub struct GetRequestBuilder<'c, C, const HAS_URL: bool>
where
    C: Client,
{
    client: &'c C,

    url: Option<Result<Url, url::ParseError>>,
}


impl<'c, C, const HAS_URL: bool> GetRequestBuilder<'c, C, HAS_URL>
where
    C: Client,
{
    pub(crate) fn new(client: &'c C) -> GetRequestBuilder<'c, C, false> {
        GetRequestBuilder { client, url: None }
    }


    pub fn endpoint_url<U>(self, relative_endpoint_path: U) -> GetRequestBuilder<'c, C, true>
    where
        U: AsRef<str>,
    {
        GetRequestBuilder {
            client: self.client,
            url: Some(build_request_url(
                self.client.server(),
                relative_endpoint_path.as_ref(),
                UrlBuildType::UnderBaseApiPath,
            )),
        }
    }

    /// Same as [`Self::endpoint_url`], but does not prepend the server's base URL
    /// (e.g. `/api/v1`) to `relative_endpoint_url`.
    pub fn endpoint_url_without_base_path<U>(
        self,
        relative_endpoint_path: U,
    ) -> GetRequestBuilder<'c, C, true>
    where
        U: AsRef<str>,
    {
        GetRequestBuilder {
            client: self.client,
            url: Some(build_request_url(
                self.client.server(),
                relative_endpoint_path.as_ref(),
                UrlBuildType::WithoutBaseApiPath,
            )),
        }
    }

    pub fn endpoint_url_with_parameters<U, P, K, V>(
        self,
        relative_endpoint_url: U,
        parameters: P,
    ) -> GetRequestBuilder<'c, C, true>
    where
        U: AsRef<str>,
        P: IntoIterator,
        P::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        GetRequestBuilder {
            client: self.client,
            url: Some(build_request_url_with_parameters(
                self.client.server(),
                relative_endpoint_url.as_ref(),
                UrlBuildType::UnderBaseApiPath,
                parameters,
            )),
        }
    }
}


impl<'c, C> GetRequestBuilder<'c, C, true>
where
    C: Client,
{
    pub(crate) fn prepare_request(self) -> ClientResult<PreparedGetRequest<'c, C>> {
        // PANIC SAFETY: `self.url` is `Some` when const generic `HAS_URL` is `true`.
        let request_url = match self.url.unwrap() {
            Ok(request_url) => request_url,
            Err(url_parse_error) => {
                return Err(ClientError::UrlPreparationError {
                    error: url_parse_error,
                })
            }
        };

        Ok(PreparedGetRequest {
            client: self.client,
            request_url,
            additional_headers: HeaderMap::new(),
        })
    }
}


impl<HC> GetRequestBuilder<'_, HC, true>
where
    HC: Client + UnauthenticatedHttpClient,
{
    pub async fn send_unauthenticated(self) -> ClientResult<ServerResponse> {
        self.prepare_request()?.execute_unauthenticated().await
    }
}

impl<HC> GetRequestBuilder<'_, HC, true>
where
    HC: Client + AuthenticatedHttpClient,
{
    pub async fn send_authenticated(self) -> ClientResult<ServerResponse> {
        self.prepare_request()?.execute_authenticated().await
    }
}
