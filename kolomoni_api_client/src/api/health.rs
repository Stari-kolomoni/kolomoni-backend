use kolomoni_core::api_models::PingResponse;
use reqwest::StatusCode;
use thiserror::Error;

use crate::{
    api::EndpointGroup,
    client::{errors::RequestError, KolomoniHttpClient},
    parsing::unexpected_response,
    request::{
        typed::{BoundTypedRequest, IntoBoundTypedRequest},
        ToRequestBuilder,
    },
    response::{raw::RawResponse, ResponseValueError},
};

#[derive(Debug, Error)]
pub enum PingError {
    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for PingError {
    fn from_request_error(client_error: RequestError) -> Self {
        Self::RequestError(client_error)
    }
}

#[inline]
pub(super) fn ping_request<'c, C>(client: &'c C) -> BoundTypedRequest<'c, C, bool, PingError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let ping_response = response.into_json_body::<PingResponse>().await?;

            Ok(ping_response.ok)
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .get()
        .endpoint_url("/health/ping")
        .build_request()
        .into_bound_typed_request(response_parser)
}



pub trait HealthAnonymousEndpoints<'c, C>: EndpointGroup<'c, C>
where
    C: KolomoniHttpClient,
{
    #[inline(always)]
    fn ping(&'c self) -> BoundTypedRequest<'c, C, bool, PingError> {
        ping_request(self.client())
    }
}

pub struct HealthApi<'c, C>
where
    C: KolomoniHttpClient,
{
    client: &'c C,
}

impl<'c, C> HealthApi<'c, C>
where
    C: KolomoniHttpClient,
{
    #[inline(always)]
    pub(crate) fn new(client: &'c C) -> Self {
        Self { client }
    }
}

impl<'c, C> EndpointGroup<'c, C> for HealthApi<'c, C>
where
    C: KolomoniHttpClient,
{
    fn client(&'c self) -> &'c C {
        self.client
    }
}

impl<'c, C> HealthAnonymousEndpoints<'c, C> for HealthApi<'c, C> where C: KolomoniHttpClient {}
