use reqwest::header::{self, HeaderMap, HeaderValue};
use serde::Serialize;
use url::Url;

use super::{build_request_url, UrlBuildType};
use crate::{
    errors::{ClientError, ClientResult},
    response::ServerResponse,
    ApiClient,
};



pub struct PostRequestBuilder<'c, HC, const HAS_URL: bool>
where
    HC: ApiClient,
{
    client: &'c HC,

    url: Option<Result<Url, url::ParseError>>,

    body: Option<Result<Vec<u8>, serde_json::Error>>,

    headers: HeaderMap,
}


impl<'c, HC, const HAS_URL: bool> PostRequestBuilder<'c, HC, HAS_URL>
where
    HC: ApiClient,
{
    pub(crate) fn new(client: &'c HC) -> PostRequestBuilder<'c, HC, false> {
        PostRequestBuilder {
            client,
            url: None,
            body: None,
            headers: HeaderMap::new(),
        }
    }

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

    /// Same as [`Self::endpoint_url`], but does not prepend the server's base URL (`/api/v1`)
    /// to `relative_endpoint_url`.
    pub fn raw_endpoint_url<U>(self, relative_endpoint_path: U) -> PostRequestBuilder<'c, HC, true>
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

impl<'c, HC> PostRequestBuilder<'c, HC, true>
where
    HC: ApiClient,
{
    pub async fn send(self) -> ClientResult<ServerResponse> {
        // PANIC SAFETY: `url` field is `Some` when `HasUrl` const generic is `true`.
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


        self.client.post(request_url, self.headers, body).await
    }
}
