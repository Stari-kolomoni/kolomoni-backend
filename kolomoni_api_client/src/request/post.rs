use serde::Serialize;
use url::Url;

use super::build_request_url;
use crate::{ClientError, ClientResult, HttpClient, ServerResponse};



pub(crate) struct PostRequestBuilder<'c, HC, const HasUrl: bool>
where
    HC: HttpClient,
{
    client: &'c HC,

    url: Option<Result<Url, url::ParseError>>,

    body: Option<Result<Vec<u8>, serde_json::Error>>,
}


impl<'c, HC, const HasUrl: bool> PostRequestBuilder<'c, HC, HasUrl>
where
    HC: HttpClient,
{
    pub(crate) fn new(client: &'c HC) -> PostRequestBuilder<'c, HC, false> {
        PostRequestBuilder {
            client,
            url: None,
            body: None,
        }
    }

    pub(crate) fn endpoint_url<U>(self, relative_endpoint_url: U) -> PostRequestBuilder<'c, HC, true>
    where
        U: AsRef<str>,
    {
        PostRequestBuilder {
            client: self.client,
            url: Some(build_request_url(
                self.client.server(),
                relative_endpoint_url.as_ref(),
            )),
            body: self.body,
        }
    }

    pub(crate) fn json<V>(self, data: &V) -> PostRequestBuilder<'c, HC, HasUrl>
    where
        V: Serialize,
    {
        let serialized_data = serde_json::to_vec(data);

        PostRequestBuilder {
            client: self.client,
            url: self.url,
            body: Some(serialized_data),
        }
    }
}

impl<'c, HC> PostRequestBuilder<'c, HC, true>
where
    HC: HttpClient,
{
    pub(crate) async fn send(self) -> ClientResult<ServerResponse> {
        // PANIC SAFETY: `url` field is `Some` when `HasUrl` const generic is `true`.
        let request_url = match self.url.unwrap() {
            Ok(request_url) => request_url,
            Err(url_parse_error) => {
                return Err(ClientError::UrlParseError {
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


        self.client.post(request_url, body).await
    }
}
