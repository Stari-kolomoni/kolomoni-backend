use std::borrow::Borrow;

use reqwest::header::{self, HeaderMap, HeaderValue};
use serde::Serialize;
use url::Url;

use crate::{
    client::{errors::RequestPreparationError, KolomoniHttpClient},
    request::{
        build_request_url,
        build_request_url_with_parameters,
        raw::{BoundRawRequest, IntoRawRequest},
        UrlBuildType,
    },
};




pub struct DeleteRequestBuilder<'c, C, const HAS_URL: bool>
where
    C: KolomoniHttpClient,
{
    client: &'c C,

    url: Option<Result<Url, url::ParseError>>,

    body: Option<Result<Vec<u8>, serde_json::Error>>,

    headers: HeaderMap,
}

impl<'c, C, const HAS_URL: bool> DeleteRequestBuilder<'c, C, HAS_URL>
where
    C: KolomoniHttpClient,
{
    pub(crate) fn new(client: &'c C) -> DeleteRequestBuilder<'c, C, false> {
        DeleteRequestBuilder {
            client,
            url: None,
            body: None,
            headers: HeaderMap::new(),
        }
    }

    pub fn endpoint_url<U>(self, relative_endpoint_url: U) -> DeleteRequestBuilder<'c, C, true>
    where
        U: AsRef<str>,
    {
        DeleteRequestBuilder {
            client: self.client,
            url: Some(build_request_url(
                self.client.api_server(),
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
    ) -> DeleteRequestBuilder<'c, C, true>
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
                self.client.api_server(),
                relative_endpoint_url.as_ref(),
                UrlBuildType::UnderBaseApiPath,
                parameters,
            )),
            body: self.body,
            headers: self.headers,
        }
    }

    pub fn json<V>(self, data: &V) -> DeleteRequestBuilder<'c, C, HAS_URL>
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
    C: KolomoniHttpClient,
{
    pub(crate) fn build_request(self) -> BoundRawRequest<'c, C> {
        // PANIC SAFETY: `self.url` is always `Some`
        // when const generic `HAS_URL` is `true`.
        let request_url = match self.url.unwrap() {
            Ok(request_url) => request_url,
            Err(url_parse_error) => {
                return BoundRawRequest::new_err(
                    self.client,
                    RequestPreparationError::UrlPreparation {
                        error: url_parse_error,
                    },
                )
            }
        };

        let raw_request = self
            .client
            .http_client()
            .delete(request_url)
            .headers(self.headers)
            .build()
            .into_raw_request();

        BoundRawRequest::new(self.client, raw_request)
    }
}
