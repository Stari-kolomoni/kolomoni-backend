use std::borrow::Borrow;

use reqwest::header::{self, HeaderMap, HeaderValue};
use serde::Serialize;
use url::Url;

use super::{build_request_url, build_request_url_with_parameters, UrlBuildType};
use crate::{
    errors::{ClientError, ClientResult},
    response::ServerResponse,
    AuthenticatedHttpClient,
    Client,
    UnauthenticatedHttpClient,
};


pub(crate) struct PreparedDeleteRequest<'c, C>
where
    C: Client,
{
    client: &'c C,

    request_url: Url,

    request_body: Option<Vec<u8>>,

    request_additional_headers: HeaderMap,
}

impl<C> PreparedDeleteRequest<'_, C>
where
    C: Client + UnauthenticatedHttpClient,
{
    async fn execute_unauthenticated(self) -> ClientResult<ServerResponse> {
        UnauthenticatedHttpClient::delete(
            self.client,
            self.request_url,
            self.request_additional_headers,
            self.request_body,
        )
        .await
    }
}

impl<C> PreparedDeleteRequest<'_, C>
where
    C: Client + AuthenticatedHttpClient,
{
    async fn execute_authenticated(self) -> ClientResult<ServerResponse> {
        AuthenticatedHttpClient::delete(
            self.client,
            self.request_url,
            self.request_additional_headers,
            self.request_body,
        )
        .await
    }
}



pub struct DeleteRequestBuilder<'c, HC, const HAS_URL: bool>
where
    HC: Client,
{
    client: &'c HC,

    url: Option<Result<Url, url::ParseError>>,

    body: Option<Result<Vec<u8>, serde_json::Error>>,

    headers: HeaderMap,
}

impl<'c, HC, const HAS_URL: bool> DeleteRequestBuilder<'c, HC, HAS_URL>
where
    HC: Client,
{
    pub(crate) fn new(client: &'c HC) -> DeleteRequestBuilder<'c, HC, false> {
        DeleteRequestBuilder {
            client,
            url: None,
            body: None,
            headers: HeaderMap::new(),
        }
    }

    pub fn endpoint_url<U>(self, relative_endpoint_url: U) -> DeleteRequestBuilder<'c, HC, true>
    where
        U: AsRef<str>,
    {
        DeleteRequestBuilder {
            client: self.client,
            url: Some(build_request_url(
                self.client.server(),
                relative_endpoint_url.as_ref(),
                UrlBuildType::UnderBaseApiPath,
            )),
            body: self.body,
            headers: self.headers,
        }
    }

    pub fn endpoint_url_with_parameters<U, P, K, V>(
        self,
        relative_endpoint_url: U,
        parameters: P,
    ) -> DeleteRequestBuilder<'c, HC, true>
    where
        U: AsRef<str>,
        P: IntoIterator,
        P::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        DeleteRequestBuilder {
            client: self.client,
            url: Some(build_request_url_with_parameters(
                self.client.server(),
                relative_endpoint_url.as_ref(),
                UrlBuildType::UnderBaseApiPath,
                parameters,
            )),
            body: self.body,
            headers: self.headers,
        }
    }

    pub fn json<V>(self, data: &V) -> DeleteRequestBuilder<'c, HC, HAS_URL>
    where
        V: Serialize,
    {
        let serialized_data = serde_json::to_vec(data);

        let mut headers = self.headers;
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
        );

        DeleteRequestBuilder {
            client: self.client,
            url: self.url,
            body: Some(serialized_data),
            headers,
        }
    }
}


impl<'c, C> DeleteRequestBuilder<'c, C, true>
where
    C: Client,
{
    pub(crate) fn prepare_request(self) -> ClientResult<PreparedDeleteRequest<'c, C>> {
        // PANIC SAFETY: `self.url` is `Some` when const generic `HAS_URL` is `true`.
        let request_url = match self.url.unwrap() {
            Ok(request_url) => request_url,
            Err(url_parse_error) => {
                return Err(ClientError::UrlPreparationError {
                    error: url_parse_error,
                })
            }
        };

        let body = match self.body {
            Some(body_data_encoding_result) => match body_data_encoding_result {
                Ok(body_data) => Some(body_data),
                Err(error) => return Err(ClientError::RequestBodySerializationError { error }),
            },
            None => None,
        };


        Ok(PreparedDeleteRequest {
            client: self.client,
            request_url,
            request_body: body,
            request_additional_headers: self.headers,
        })
    }
}


impl<HC> DeleteRequestBuilder<'_, HC, true>
where
    HC: Client + UnauthenticatedHttpClient,
{
    pub async fn send_unauthenticated(self) -> ClientResult<ServerResponse> {
        self.prepare_request()?.execute_unauthenticated().await
    }
}

impl<HC> DeleteRequestBuilder<'_, HC, true>
where
    HC: Client + AuthenticatedHttpClient,
{
    pub async fn send_authenticated(self) -> ClientResult<ServerResponse> {
        self.prepare_request()?.execute_authenticated().await
    }
}
