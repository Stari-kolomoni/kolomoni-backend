use kolomoni_core::api_models::PingResponse;

use crate::{request::RequestBuilder, Client, ClientResult};

pub struct HealthApi<'c> {
    client: &'c Client,
}

impl<'c> HealthApi<'c> {
    pub async fn ping(&self) -> ClientResult<bool> {
        let response = RequestBuilder::get(self.client)
            .endpoint_url("/health/ping")
            .send()
            .await?;

        let response_body: PingResponse = response.json().await?;

        Ok(response_body.ok)
    }
}
