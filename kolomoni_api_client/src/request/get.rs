use std::borrow::Borrow;

use reqwest::header::HeaderMap;
use url::Url;

use super::{build_request_url, build_request_url_with_parameters, UrlBuildType};
use crate::{
    errors::{ClientError, ClientResult},
    response::ServerResponse,
    ApiClient,
};

pub struct GetRequestBuilder<'c, HC, const HAS_URL: bool>
where
    HC: ApiClient,
{
    client: &'c HC,
    url: Option<Result<Url, url::ParseError>>,
}

impl<'c, HC, const HAS_URL: bool> GetRequestBuilder<'c, HC, HAS_URL>
where
    HC: ApiClient,
{
    pub(crate) fn new(client: &'c HC) -> GetRequestBuilder<'c, HC, false> {
        GetRequestBuilder { client, url: None }
    }

    pub fn endpoint_url<U>(self, relative_endpoint_path: U) -> GetRequestBuilder<'c, HC, true>
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

    /// Same as [`Self::endpoint_url`], but does not prepend the server's base URL (`/api/v1`)
    /// to `relative_endpoint_url`.
    pub fn raw_endpoint_url<U>(self, relative_endpoint_path: U) -> GetRequestBuilder<'c, HC, true>
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
                UrlBuildType::UnderBaseApiPath,
                parameters,
            )),
        }
    }
}

impl<'c, HC> GetRequestBuilder<'c, HC, true>
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

        self.client.get(request_url, HeaderMap::new()).await
    }
}
