use kolomoni_core::api_models::PingResponse;

use crate::{urls::build_request_url, Client, ClientError, ClientResult};

pub struct HealthApi<'c> {
    client: &'c Client,
}

impl<'c> HealthApi<'c> {
    pub async fn ping(&self) -> ClientResult<bool> {
        let response = self
            .client
            .get(build_request_url(
                &self.client.server,
                "/health/ping",
            )?)
            .await?;


        let response_body: PingResponse = response
            .json()
            .await
            .map_err(|error| ClientError::ResponseJsonBodyError { error })?;

        Ok(response_body.ok)
    }
}
