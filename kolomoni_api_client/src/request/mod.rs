use std::borrow::Borrow;

use delete::DeleteRequestBuilder;
use get::GetRequestBuilder;
use patch::PatchRequestBuilder;
use post::PostRequestBuilder;
use url::Url;

use crate::{server::ApiServer, HttpClient};

pub(crate) mod delete;
pub(crate) mod get;
pub(crate) mod patch;
pub(crate) mod post;


pub(crate) struct RequestBuilder;

impl RequestBuilder {
    pub(crate) fn get<'c, HC>(client: &'c HC) -> GetRequestBuilder<'c, HC, false>
    where
        HC: HttpClient,
    {
        GetRequestBuilder::<'c, HC, false>::new(client)
    }

    pub(crate) fn post<'c, HC>(client: &'c HC) -> PostRequestBuilder<'c, HC, false>
    where
        HC: HttpClient,
    {
        PostRequestBuilder::<'c, HC, false>::new(client)
    }

    pub(crate) fn patch<'c, HC>(client: &'c HC) -> PatchRequestBuilder<'c, HC, false>
    where
        HC: HttpClient,
    {
        PatchRequestBuilder::<'c, HC, false>::new(client)
    }

    pub(crate) fn delete<'c, HC>(client: &'c HC) -> DeleteRequestBuilder<'c, HC, false>
    where
        HC: HttpClient,
    {
        DeleteRequestBuilder::<'c, HC, false>::new(client)
    }
}


fn build_request_url(server: &ApiServer, endpoint: &str) -> Result<Url, url::ParseError> {
    if !endpoint.starts_with('/') {
        Url::parse(&format!("{}/{}", server.base_url(), endpoint))
    } else {
        Url::parse(&format!("{}{}", server.base_url(), endpoint))
    }
}

fn build_request_url_with_parameters<P, K, V>(
    server: &ApiServer,
    endpoint: &str,
    parameters: P,
) -> Result<Url, url::ParseError>
where
    P: IntoIterator,
    P::Item: Borrow<(K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
{
    if !endpoint.starts_with('/') {
        Url::parse_with_params(
            &format!("{}/{}", server.base_url(), endpoint),
            parameters,
        )
    } else {
        Url::parse_with_params(
            &format!("{}{}", server.base_url(), endpoint),
            parameters,
        )
    }
}
