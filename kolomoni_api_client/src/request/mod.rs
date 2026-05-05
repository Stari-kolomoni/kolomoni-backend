use std::borrow::Borrow;

use url::Url;

use crate::{
    client::KolomoniHttpClient,
    request::{
        builder::{
            DeleteRequestBuilder,
            GetRequestBuilder,
            PatchRequestBuilder,
            PostRequestBuilder,
        },
        raw::BoundRawRequest,
    },
    server::KolomoniApiServer,
};

pub mod builder;
pub mod raw;
pub mod typed;


/// Indicates how the URL should be built:
/// either the `/api/v1/` prefix (i.e. the "base API path")
/// should be prefixed to the given endpoint, or not.
///
/// The latter is sometimes useful if you want to call arbitrary
/// API endpoints, but most of the crate will use the
/// automatic API V1 prefix.
enum UrlBuildType {
    WithoutBaseApiPath,
    UnderBaseApiPath,
}


/// Returns a [`Url`] pointing to the required `endpoint` on the server,
/// or [`url::ParseError`] if there was an error while constructing the URL.
fn build_request_url(
    server: &KolomoniApiServer,
    endpoint: &str,
    build_type: UrlBuildType,
) -> Result<Url, url::ParseError> {
    let endpoint_without_leading_slash = endpoint.strip_prefix('/').unwrap_or(endpoint);

    let url_to_build_on = match build_type {
        UrlBuildType::WithoutBaseApiPath => server.url_without_base_api_path(),
        UrlBuildType::UnderBaseApiPath => server.url_with_base_api_path(),
    };

    url_to_build_on.join(endpoint_without_leading_slash)
}


/// Returns a [`Url`] pointing to the required `endpoint` on the server,
/// or [`url::ParseError`] if there was an error while constructing the URL.
///
/// Allows adding URL `parameters`.
fn build_request_url_with_parameters<P, K, V>(
    server: &KolomoniApiServer,
    endpoint: &str,
    build_type: UrlBuildType,
    parameters: P,
) -> Result<Url, url::ParseError>
where
    P: IntoIterator,
    P::Item: Borrow<(K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
{
    let endpoint_without_leading_slash = endpoint.strip_prefix('/').unwrap_or(endpoint);

    let url_to_build_on = match build_type {
        UrlBuildType::WithoutBaseApiPath => server.url_without_base_api_path(),
        UrlBuildType::UnderBaseApiPath => server.url_with_base_api_path(),
    };


    let mut full_url = url_to_build_on.join(endpoint_without_leading_slash)?;

    {
        let mut full_url_params = full_url.query_pairs_mut();

        for item in parameters.into_iter() {
            let (name, value) = item.borrow();
            let name = name.as_ref();
            let value = value.as_ref();

            full_url_params.append_pair(name, value);
        }
    }

    Ok(full_url)
}


/// All [`KolomoniHttpClient`]s implement this trait, allowing
/// us to build HTTP requests elegantly (e.g. by just doing
/// `client.get()` and getting back a builder),
pub trait ToRequestBuilder: KolomoniHttpClient {
    /// Constructs a HTTP GET request builder.
    fn get<'c>(&'c self) -> GetRequestBuilder<'c, Self, false>
    where
        Self: Sized;

    /// Constructs a HTTP POST request builder.
    fn post<'c>(&'c self) -> PostRequestBuilder<'c, Self, false>
    where
        Self: Sized;

    /// Constructs a HTTP PATCH request builder.
    fn patch<'c>(&'c self) -> PatchRequestBuilder<'c, Self, false>
    where
        Self: Sized;

    /// Constructs a HTTP DELETE request builder.
    fn delete<'c>(&'c self) -> DeleteRequestBuilder<'c, Self, false>
    where
        Self: Sized;
}

impl<C> ToRequestBuilder for C
where
    C: KolomoniHttpClient,
{
    fn get<'c>(&'c self) -> GetRequestBuilder<'c, Self, false>
    where
        Self: Sized,
    {
        GetRequestBuilder::<'c, Self, false>::new(self)
    }

    fn post<'c>(&'c self) -> PostRequestBuilder<'c, Self, false>
    where
        Self: Sized,
    {
        PostRequestBuilder::<'c, Self, false>::new(self)
    }

    fn patch<'c>(&'c self) -> PatchRequestBuilder<'c, Self, false>
    where
        Self: Sized,
    {
        PatchRequestBuilder::<'c, Self, false>::new(self)
    }

    fn delete<'c>(&'c self) -> DeleteRequestBuilder<'c, Self, false>
    where
        Self: Sized,
    {
        DeleteRequestBuilder::<'c, Self, false>::new(self)
    }
}
