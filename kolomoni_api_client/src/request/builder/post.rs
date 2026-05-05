use reqwest::header::{self, HeaderMap, HeaderValue};
use serde::Serialize;
use url::Url;

use crate::{
    client::{errors::RequestPreparationError, KolomoniHttpClient},
    request::{build_request_url, raw::IntoRawRequest, BoundRawRequest, UrlBuildType},
};



pub struct PostRequestBuilder<'c, C, const HAS_URL: bool>
where
    C: KolomoniHttpClient,
{
    client: &'c C,

    url: Option<Result<Url, url::ParseError>>,

    body: Option<Result<Vec<u8>, serde_json::Error>>,

    headers: HeaderMap,
}


impl<'c, C, const HAS_URL: bool> PostRequestBuilder<'c, C, HAS_URL>
where
    C: KolomoniHttpClient,
{
    pub(crate) fn new(client: &'c C) -> PostRequestBuilder<'c, C, false> {
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
    pub fn endpoint_url<U>(self, relative_endpoint_path: U) -> PostRequestBuilder<'c, C, true>
    where
        U: AsRef<str>,
    {
        PostRequestBuilder {
            client: self.client,
            url: Some(build_request_url(
                self.client.api_server(),
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
    ) -> PostRequestBuilder<'c, C, true>
    where
        U: AsRef<str>,
    {
        PostRequestBuilder {
            client: self.client,
            url: Some(build_request_url(
                self.client.api_server(),
                relative_endpoint_path.as_ref(),
                UrlBuildType::WithoutBaseApiPath,
            )),
            body: self.body,
            headers: self.headers,
        }
    }

    pub fn json<V>(self, data: &V) -> PostRequestBuilder<'c, C, HAS_URL>
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
    C: KolomoniHttpClient,
{
    pub fn build_request(self) -> BoundRawRequest<'c, C> {
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
            .post(request_url)
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
