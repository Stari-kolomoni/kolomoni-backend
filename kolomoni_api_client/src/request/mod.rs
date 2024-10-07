use get::GetRequestBuilder;
use post::PostRequestBuilder;
use url::Url;

use crate::{server::ApiServer, HttpClient};

pub(crate) mod get;
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
}


fn build_request_url(server: &ApiServer, endpoint: &str) -> Result<Url, url::ParseError> {
    if !endpoint.starts_with('/') {
        Url::parse(&format!("{}/{}", server.base_url(), endpoint))
    } else {
        Url::parse(&format!("{}{}", server.base_url(), endpoint))
    }
}
