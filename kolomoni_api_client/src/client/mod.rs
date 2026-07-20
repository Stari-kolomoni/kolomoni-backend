use std::{future::Future, pin::Pin};


pub mod errors;
pub use errors::{ClientInitializationError, RequestError};

mod authenticated;
pub use authenticated::AuthenticatedKolomoniClient;

mod unauthenticated;
pub use unauthenticated::{UnauthenticatedClientOptions, UnauthenticatedKolomoniClient};

#[cfg(feature = "unrestricted_client")]
mod unrestricted;
#[cfg(feature = "unrestricted_client")]
pub use unrestricted::UnrestrictedUnauthenticatedKolomoniClient;

use crate::{
    client::errors::RequestResult,
    request::raw::RawRequest,
    response::raw::RawResponse,
    server::KolomoniApiServer,
};



/// A basic HTTP functionality over HTTP.
pub trait KolomoniHttpClient {
    fn http_client(&self) -> &reqwest::Client;

    fn api_server(&self) -> &KolomoniApiServer;

    /// The implementor MAY modify the provided `request`
    /// before executing it (for example, to add an
    /// authentication header), depending on the situation.
    ///
    /// See also: [`RawRequest::authentication_override`]
    fn send<'a>(
        &'a self,
        request: RawRequest,
    ) -> Pin<Box<dyn Future<Output = RequestResult<RawResponse>> + Send + 'a>>;
}


/// This is the default User-Agent we declare when using any of the clients this crate implements.
const DEFAULT_CLIENT_USER_AGENT: &str = concat!(
    "kolomoni_api_client / v",
    env!("CARGO_PKG_VERSION")
);


/// Given a [`RawRequest`] (which, unlike a normal [`reqwest::Request`], bubbles up request
/// construction errors) and a [`reqwest::Request`], this function executes
/// the request and returns the response wrapped in [`RawResponse`] and better error handling.
#[inline]
async fn execute_prepared_raw_request(
    reqwest_client: &reqwest::Client,
    raw_request: RawRequest,
) -> RequestResult<RawResponse> {
    let request = match raw_request.try_into_inner() {
        Ok(request) => request,
        Err(preparation_error) => return Err(preparation_error.into_full_request_error()),
    };

    reqwest_client
        .execute(request)
        .await
        .map(RawResponse::new)
        .map_err(|error| RequestError::RequestExecutionError { error })
}
