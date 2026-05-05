use std::borrow::Borrow;

use url::Url;

use crate::{
    client::{errors::RequestPreparationError, KolomoniHttpClient},
    request::{
        build_request_url,
        build_request_url_with_parameters,
        raw::IntoRawRequest,
        BoundRawRequest,
        UrlBuildType,
    },
};


pub struct GetRequestBuilder<'c, C, const HAS_URL: bool>
where
    C: KolomoniHttpClient,
{
    client: &'c C,

    url: Option<Result<Url, url::ParseError>>,
}


impl<'c, C, const HAS_URL: bool> GetRequestBuilder<'c, C, HAS_URL>
where
    C: KolomoniHttpClient,
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
                self.client.api_server(),
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
                self.client.api_server(),
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
                self.client.api_server(),
                relative_endpoint_url.as_ref(),
                UrlBuildType::UnderBaseApiPath,
                parameters,
            )),
        }
    }
}


impl<'c, C> GetRequestBuilder<'c, C, true>
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

        let raw_request = self
            .client
            .http_client()
            .get(request_url)
            .build()
            .into_raw_request();

        BoundRawRequest::new(self.client, raw_request)
    }
}
