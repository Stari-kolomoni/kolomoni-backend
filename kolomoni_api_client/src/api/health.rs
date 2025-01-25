use std::future::Future;

use kolomoni_core::api_models::PingResponse;

use crate::{
    errors::ClientResult,
    request::RequestBuilder,
    ApiClient,
    AuthenticatedApiClient,
    UnauthenticatedApiClient,
};


pub trait SharedHealthEndpoints {
    fn ping(&self) -> impl Future<Output = ClientResult<bool>>;
}


async fn ping<C>(client: &C) -> ClientResult<bool>
where
    C: ApiClient,
{
    let response = RequestBuilder::get(client)
        .endpoint_url("/health/ping")
        .send()
        .await?;

    let response_body: PingResponse = response.json().await?;

    Ok(response_body.ok)
}



pub struct HealthUnauthenticatedApi<'c, C>
where
    C: UnauthenticatedApiClient,
{
    client: &'c C,
}

impl<'c, C> HealthUnauthenticatedApi<'c, C>
where
    C: UnauthenticatedApiClient,
{
    pub(crate) const fn new(client: &'c C) -> Self {
        Self { client }
    }
}

impl<'c, C> SharedHealthEndpoints for HealthUnauthenticatedApi<'c, C>
where
    C: UnauthenticatedApiClient,
{
    async fn ping(&self) -> ClientResult<bool> {
        ping(self.client).await
    }
}



pub struct HealthAuthenticatedApi<'c, C>
where
    C: AuthenticatedApiClient,
{
    client: &'c C,
}

impl<'c, C> HealthAuthenticatedApi<'c, C>
where
    C: AuthenticatedApiClient,
{
    pub(crate) const fn new(client: &'c C) -> Self {
        Self { client }
    }
}


impl<'c, C> SharedHealthEndpoints for HealthAuthenticatedApi<'c, C>
where
    C: AuthenticatedApiClient,
{
    async fn ping(&self) -> ClientResult<bool> {
        ping(self.client).await
    }
}
