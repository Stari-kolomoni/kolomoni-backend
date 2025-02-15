use std::future::Future;

use kolomoni_core::api_models::PingResponse;

use crate::{
    errors::ClientResult,
    request::RequestBuilder,
    AuthenticatedHttpClient,
    Client,
    UnauthenticatedHttpClient,
};


pub trait SharedHealthEndpoints {
    fn ping(&self) -> impl Future<Output = ClientResult<bool>>;
}


async fn ping<C>(client: &C) -> ClientResult<bool>
where
    C: UnauthenticatedHttpClient,
{
    let response = RequestBuilder::get(client)
        .endpoint_url("/health/ping")
        .send_unauthenticated()
        .await?;

    let response_body: PingResponse = response.json().await?;

    Ok(response_body.ok)
}



pub struct HealthUnauthenticatedApi<'c, C>
where
    C: Client + UnauthenticatedHttpClient,
{
    client: &'c C,
}

impl<'c, C> HealthUnauthenticatedApi<'c, C>
where
    C: Client + UnauthenticatedHttpClient,
{
    pub(crate) const fn new(client: &'c C) -> Self {
        Self { client }
    }
}

impl<'c, C> SharedHealthEndpoints for HealthUnauthenticatedApi<'c, C>
where
    C: Client + UnauthenticatedHttpClient,
{
    async fn ping(&self) -> ClientResult<bool> {
        ping(self.client).await
    }
}



pub struct HealthAuthenticatedApi<'c, C>
where
    C: Client + UnauthenticatedHttpClient + AuthenticatedHttpClient,
{
    client: &'c C,
}

impl<'c, C> HealthAuthenticatedApi<'c, C>
where
    C: Client + UnauthenticatedHttpClient + AuthenticatedHttpClient,
{
    pub(crate) const fn new(client: &'c C) -> Self {
        Self { client }
    }
}


impl<'c, C> SharedHealthEndpoints for HealthAuthenticatedApi<'c, C>
where
    C: Client + UnauthenticatedHttpClient + AuthenticatedHttpClient,
{
    async fn ping(&self) -> ClientResult<bool> {
        ping(self.client).await
    }
}
