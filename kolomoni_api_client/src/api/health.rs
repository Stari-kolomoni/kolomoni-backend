use kolomoni_core::api_models::PingResponse;

use crate::{
    errors::ClientResult,
    request::RequestBuilder,
    AuthenticatedClient,
    Client,
    HttpClient,
};


async fn ping<C>(client: &C) -> ClientResult<bool>
where
    C: HttpClient,
{
    let response = RequestBuilder::get(client)
        .endpoint_url("/health/ping")
        .send()
        .await?;

    let response_body: PingResponse = response.json().await?;

    Ok(response_body.ok)
}


pub struct HealthApi<'c> {
    client: &'c Client,
}

impl<'c> HealthApi<'c> {
    pub(crate) const fn new(client: &'c Client) -> Self {
        Self { client }
    }

    pub async fn ping(&self) -> ClientResult<bool> {
        ping(self.client).await
    }
}


pub struct HealthAuthenticatedApi<'c> {
    client: &'c AuthenticatedClient,
}

impl<'c> HealthAuthenticatedApi<'c> {
    pub(crate) const fn new(client: &'c AuthenticatedClient) -> Self {
        Self { client }
    }

    pub async fn ping(&self) -> ClientResult<bool> {
        ping(self.client).await
    }
}
