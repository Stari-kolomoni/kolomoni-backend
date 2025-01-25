use std::borrow::Borrow;

use delete::DeleteRequestBuilder;
use get::GetRequestBuilder;
use patch::PatchRequestBuilder;
use post::PostRequestBuilder;
use url::Url;

use crate::{server::ApiServer, ApiClient};

pub mod delete;
pub mod get;
pub mod patch;
pub mod post;


pub struct RequestBuilder;

impl RequestBuilder {
    pub(crate) fn get<'c, HC>(client: &'c HC) -> GetRequestBuilder<'c, HC, false>
    where
        HC: ApiClient,
    {
        GetRequestBuilder::<'c, HC, false>::new(client)
    }

    pub(crate) fn post<'c, HC>(client: &'c HC) -> PostRequestBuilder<'c, HC, false>
    where
        HC: ApiClient,
    {
        PostRequestBuilder::<'c, HC, false>::new(client)
    }

    pub(crate) fn patch<'c, HC>(client: &'c HC) -> PatchRequestBuilder<'c, HC, false>
    where
        HC: ApiClient,
    {
        PatchRequestBuilder::<'c, HC, false>::new(client)
    }

    pub(crate) fn delete<'c, HC>(client: &'c HC) -> DeleteRequestBuilder<'c, HC, false>
    where
        HC: ApiClient,
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


pub trait ApiClientRequestBuild: ApiClient {
    fn get_request_builder(&self) -> GetRequestBuilder<'_, Self, false>
    where
        Self: Sized;

    fn post_request_builder(&self) -> PostRequestBuilder<'_, Self, false>
    where
        Self: Sized;

    fn patch_request_builder(&self) -> PatchRequestBuilder<'_, Self, false>
    where
        Self: Sized;

    fn delete_request_builder(&self) -> DeleteRequestBuilder<'_, Self, false>
    where
        Self: Sized;
}

impl<C> ApiClientRequestBuild for C
where
    C: ApiClient,
{
    fn get_request_builder(&self) -> GetRequestBuilder<'_, Self, false>
    where
        Self: Sized,
    {
        RequestBuilder::get(self)
    }

    fn post_request_builder(&self) -> PostRequestBuilder<'_, Self, false>
    where
        Self: Sized,
    {
        RequestBuilder::post(self)
    }

    fn patch_request_builder(&self) -> PatchRequestBuilder<'_, Self, false>
    where
        Self: Sized,
    {
        RequestBuilder::patch(self)
    }

    fn delete_request_builder(&self) -> DeleteRequestBuilder<'_, Self, false>
    where
        Self: Sized,
    {
        RequestBuilder::delete(self)
    }
}
