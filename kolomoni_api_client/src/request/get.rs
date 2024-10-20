use std::borrow::Borrow;

use url::Url;

use super::{build_request_url, build_request_url_with_parameters};
use crate::{
    errors::{ClientError, ClientResult},
    response::ServerResponse,
    HttpClient,
};

pub(crate) struct GetRequestBuilder<'c, HC, const HAS_URL: bool>
where
    HC: HttpClient,
{
    client: &'c HC,
    url: Option<Result<Url, url::ParseError>>,
}

impl<'c, HC, const HAS_URL: bool> GetRequestBuilder<'c, HC, HAS_URL>
where
    HC: HttpClient,
{
    pub(crate) fn new(client: &'c HC) -> GetRequestBuilder<'c, HC, false> {
        GetRequestBuilder { client, url: None }
    }

    pub(crate) fn endpoint_url<U>(self, relative_endpoint_url: U) -> GetRequestBuilder<'c, HC, true>
    where
        U: AsRef<str>,
    {
        GetRequestBuilder {
            client: self.client,
            url: Some(build_request_url(
                self.client.server(),
                relative_endpoint_url.as_ref(),
            )),
        }
    }

    pub(crate) fn endpoint_url_with_parameters<U, P, K, V>(
        self,
        relative_endpoint_url: U,
        parameters: P,
    ) -> GetRequestBuilder<'c, HC, true>
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
                parameters,
            )),
        }
    }
}

impl<'c, HC> GetRequestBuilder<'c, HC, true>
where
    HC: HttpClient,
{
    pub(crate) async fn send(self) -> ClientResult<ServerResponse> {
        // PANIC SAFETY: `url` field is `Some` when `HasUrl` const generic is `true`.
        let request_url = match self.url.unwrap() {
            Ok(request_url) => request_url,
            Err(url_parse_error) => {
                return Err(ClientError::UrlPreparationError {
                    error: url_parse_error,
                })
            }
        };

        self.client.get(request_url).await
    }
}
