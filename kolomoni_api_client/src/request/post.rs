use reqwest::header::{self, HeaderMap, HeaderValue};
use serde::Serialize;
use url::Url;

use super::{
    build_request_url,
    PreparedAuthenticatedRequest,
    PreparedUnauthenticatedRequest,
    UrlBuildType,
};
use crate::{
    errors::{ClientError, ClientResult},
    response::ServerResponse,
    AuthenticatedHttpClient,
    Client,
    UnauthenticatedHttpClient,
};



pub(crate) struct PreparedPostRequest<'c, C>
where
    C: Client,
{
    client: &'c C,

    request_url: Url,

    request_body: Option<Vec<u8>>,

    request_additional_headers: HeaderMap,
}

impl<'c, C> PreparedUnauthenticatedRequest for PreparedPostRequest<'c, C>
where
    C: Client + UnauthenticatedHttpClient,
{
    async fn execute_unauthenticated(self) -> ClientResult<ServerResponse> {
        UnauthenticatedHttpClient::post(
            self.client,
            self.request_url,
            self.request_additional_headers,
            self.request_body,
        )
        .await
    }
}

impl<'c, C> PreparedAuthenticatedRequest for PreparedPostRequest<'c, C>
where
    C: Client + AuthenticatedHttpClient,
{
    async fn execute_authenticated(self) -> ClientResult<ServerResponse> {
        AuthenticatedHttpClient::post(
            self.client,
            self.request_url,
            self.request_additional_headers,
            self.request_body,
        )
        .await
    }
}




pub struct PostRequestBuilder<'c, HC, const HAS_URL: bool>
where
    HC: Client,
{
    client: &'c HC,

    url: Option<Result<Url, url::ParseError>>,

    body: Option<Result<Vec<u8>, serde_json::Error>>,

    headers: HeaderMap,
}


impl<'c, HC, const HAS_URL: bool> PostRequestBuilder<'c, HC, HAS_URL>
where
    HC: Client,
{
    pub(crate) fn new(client: &'c HC) -> PostRequestBuilder<'c, HC, false> {
        PostRequestBuilder {
            client,
            url: None,
            body: None,
            headers: HeaderMap::new(),
        }
    }

    /// Sets the endpoint URL based on the given `relative_endpoint_path`.
    /// The relative endpoint path is appended to the base API path (e.g.
    /// given a base API path of `/api/v1/` and a `relative_endpoint_path` of `/hello/world`,
    /// this function would set the request path to `/api/v1/hello/world`).
    pub fn endpoint_url<U>(self, relative_endpoint_path: U) -> PostRequestBuilder<'c, HC, true>
    where
        U: AsRef<str>,
    {
        PostRequestBuilder {
            client: self.client,
            url: Some(build_request_url(
                self.client.server(),
                relative_endpoint_path.as_ref(),
                UrlBuildType::UnderBaseApiPath,
            )),
            body: self.body,
            headers: self.headers,
        }
    }

    /// Sets the endpoint URL based on the given `relative_endpoint_path`.
    ///
    /// Unlike with [`endpoint_url`], the base API path is not used in the final path.
    /// Use this method when accessing endpoints that are not under the base API path.
    pub fn endpoint_url_without_base_path<U>(
        self,
        relative_endpoint_path: U,
    ) -> PostRequestBuilder<'c, HC, true>
    where
        U: AsRef<str>,
    {
        PostRequestBuilder {
            client: self.client,
            url: Some(build_request_url(
                self.client.server(),
                relative_endpoint_path.as_ref(),
                UrlBuildType::WithoutBaseApiPath,
            )),
            body: self.body,
            headers: self.headers,
        }
    }

    pub fn json<V>(self, data: &V) -> PostRequestBuilder<'c, HC, HAS_URL>
    where
        V: Serialize,
    {
        let serialized_data = serde_json::to_vec(data);

        let mut headers = self.headers;
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
        );

        PostRequestBuilder {
            client: self.client,
            url: self.url,
            body: Some(serialized_data),
            headers,
        }
    }
}

impl<'c, C> PostRequestBuilder<'c, C, true>
where
    C: Client,
{
    pub(crate) fn prepare(self) -> ClientResult<PreparedPostRequest<'c, C>> {
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


        Ok(PreparedPostRequest {
            client: self.client,
            request_url,
            request_body: body,
            request_additional_headers: self.headers,
        })
    }
}


impl<'c, HC> PostRequestBuilder<'c, HC, true>
where
    HC: Client + UnauthenticatedHttpClient,
{
    pub async fn send_unauthenticated(self) -> ClientResult<ServerResponse> {
        self.prepare()?.execute_unauthenticated().await
    }
}

impl<'c, HC> PostRequestBuilder<'c, HC, true>
where
    HC: Client + AuthenticatedHttpClient,
{
    pub async fn send_authenticated(self) -> ClientResult<ServerResponse> {
        self.prepare()?.execute_authenticated().await
    }
}
