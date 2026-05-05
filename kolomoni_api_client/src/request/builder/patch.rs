use reqwest::header::{self, HeaderMap, HeaderValue};
use serde::Serialize;
use url::Url;

use crate::{
    client::{errors::RequestPreparationError, KolomoniHttpClient},
    request::{build_request_url, raw::IntoRawRequest, BoundRawRequest, UrlBuildType},
};



pub struct PatchRequestBuilder<'c, C, const HAS_URL: bool>
where
    C: KolomoniHttpClient,
{
    client: &'c C,

    url: Option<Result<Url, url::ParseError>>,

    body: Option<Result<Vec<u8>, serde_json::Error>>,

    headers: HeaderMap,
}


impl<'c, C, const HAS_URL: bool> PatchRequestBuilder<'c, C, HAS_URL>
where
    C: KolomoniHttpClient,
{
    pub(crate) fn new(client: &'c C) -> PatchRequestBuilder<'c, C, false> {
        PatchRequestBuilder {
            client,
            url: None,
            body: None,
            headers: HeaderMap::new(),
        }
    }

    pub fn endpoint_url<U>(self, relative_endpoint_url: U) -> PatchRequestBuilder<'c, C, true>
    where
        U: AsRef<str>,
    {
        PatchRequestBuilder {
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

    pub fn json<V>(self, data: &V) -> PatchRequestBuilder<'c, C, HAS_URL>
    where
        V: Serialize,
    {
        let serialized_data = serde_json::to_vec(data);

        let mut headers = self.headers;
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
        );

        PatchRequestBuilder {
            client: self.client,
            url: self.url,
            body: Some(serialized_data),
            headers,
        }
    }
}

impl<'c, C> PatchRequestBuilder<'c, C, true>
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

        let mut raw_request = self
            .client
            .http_client()
            .patch(request_url)
            .headers(self.headers);

        if let Some(body_data_encoding_result) = self.body {
            match body_data_encoding_result {
                Ok(body_data) => {
                    raw_request = raw_request.body(body_data);
                }
                Err(error) => {
                    return BoundRawRequest::new_err(
                        self.client,
                        RequestPreparationError::RequestBodySerialization { error },
                    );
                }
            }
        }

        BoundRawRequest::new(
            self.client,
            raw_request.build().into_raw_request(),
        )
    }
}
