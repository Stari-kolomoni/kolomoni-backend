use std::str::FromStr;

use crate::{
    client::{
        errors::{RequestPreparationError, RequestResult},
        KolomoniHttpClient,
    },
    response::raw::RawResponse,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthenticationOverride {
    /// Keep authentication as-is:
    /// - if the client that will execute the underlying [`RawRequest`]
    ///   is an *unauthenticatied client*, no auth header will be included;
    /// - if the client that will execute the underlying [`RawRequest`]
    ///   is an *authenticated client*, it will append the authentication
    ///   header before actually executing the request.
    Inherit,

    /// Disables authentication.
    Disable,
}


/// A wrapper around a fallible [`reqwest::Request`] and additional request options.
///
/// The request is internally fallible, because certain request building operations
/// can fail, such as URL encoding. If you opt to wrap this into a [`TypedRequest`]
/// or [`BoundTypedRequest`], as most use-cases do, this potential request preparation
/// error will be bubbled up into that type.
pub struct RawRequest {
    inner: Result<reqwest::Request, RequestPreparationError>,
    authentication_override: AuthenticationOverride,
}

impl RawRequest {
    pub fn new(request: Result<reqwest::Request, RequestPreparationError>) -> Self {
        Self {
            inner: request,
            authentication_override: AuthenticationOverride::Inherit,
        }
    }

    pub fn try_inner(&self) -> Result<&reqwest::Request, &RequestPreparationError> {
        self.inner.as_ref()
    }

    pub fn try_inner_mut(&mut self) -> &mut Result<reqwest::Request, RequestPreparationError> {
        &mut self.inner
    }

    pub fn try_into_inner(self) -> Result<reqwest::Request, RequestPreparationError> {
        self.inner
    }

    /// Calls the closure, updating the inner [`reqwest::Request`].
    ///
    /// If the inner request wasn't successfully constructed in the first place,
    /// the closure is not called and nothing is changed.
    pub fn modify_request<F>(&mut self, map_closure: F)
    where
        F: FnOnce(reqwest::Request) -> Result<reqwest::Request, RequestPreparationError>,
    {
        // This is a fake temporary value that we use only to swap out the current `inner`
        // in order to own it. If we didn't do this, we'd need to wrap `inner` in `Option`
        // and then `Option::take` it, the checks for which would pollute the rest of the struct.
        let fake_inner = Ok(reqwest::Request::new(
            reqwest::Method::GET,
            reqwest::Url::from_str("https://home.arpa").unwrap(),
        ));

        let mut temporary_owned_inner = fake_inner;
        std::mem::swap(&mut self.inner, &mut temporary_owned_inner);

        let updated_inner = match temporary_owned_inner {
            Ok(request) => map_closure(request),
            Err(request_construction_error) => Err(request_construction_error),
        };

        self.inner = updated_inner;
    }

    pub(crate) fn authentication_override(&self) -> AuthenticationOverride {
        self.authentication_override
    }

    /// Disables any authentication that might be present or get added
    /// before executing this request. Internally, this will indicate
    /// to the underlying [`KolomoniHttpClient`] in use that it must avoid
    /// adding an authentication header. If the underlying client will
    /// already be an unauthenticated client, this will have no effect.
    pub fn without_authentication(mut self) -> Self {
        self.authentication_override = AuthenticationOverride::Disable;
        self
    }

    #[inline]
    pub async fn send<C>(self, client: &C) -> RequestResult<RawResponse>
    where
        C: KolomoniHttpClient,
    {
        client.send(self).await
    }
}


/// Converts `self` into a [`RawRequest`] infallibly.
///
/// Notably, this is implemented on `Result<reqwest::Request, reqwest::Error>`
/// to allow chaining.
pub trait IntoRawRequest {
    fn into_raw_request(self) -> RawRequest;
}

impl IntoRawRequest for Result<reqwest::Request, reqwest::Error> {
    fn into_raw_request(self) -> RawRequest {
        RawRequest::new(
            self.map_err(|error| RequestPreparationError::RequestInstancePreparation { error }),
        )
    }
}




/// A version of [`RawRequest`] that additionally
/// captures the `C: `[`KolomoniHttpClient`] that will be
/// used to execute the request.
pub struct BoundRawRequest<'c, C>
where
    C: KolomoniHttpClient,
{
    pub(super) client: &'c C,
    pub(super) request: RawRequest,
}

impl<'c, C> BoundRawRequest<'c, C>
where
    C: KolomoniHttpClient,
{
    pub(crate) fn new(client: &'c C, request: RawRequest) -> Self {
        Self { client, request }
    }

    pub(crate) fn new_err(client: &'c C, error: RequestPreparationError) -> Self {
        Self {
            client,
            request: RawRequest::new(Err(error)),
        }
    }

    #[inline]
    pub async fn send(self) -> RequestResult<RawResponse> {
        self.client.send(self.request).await
    }
}

impl<'c, C> AsRef<RawRequest> for BoundRawRequest<'c, C>
where
    C: KolomoniHttpClient,
{
    fn as_ref(&self) -> &RawRequest {
        &self.request
    }
}

impl<'c, C> AsMut<RawRequest> for BoundRawRequest<'c, C>
where
    C: KolomoniHttpClient,
{
    fn as_mut(&mut self) -> &mut RawRequest {
        &mut self.request
    }
}
