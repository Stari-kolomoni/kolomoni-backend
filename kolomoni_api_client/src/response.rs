use kolomoni_core::api_models::{
    CategoryErrorReason,
    ErrorReason,
    ResponseWithErrorReason,
    WordErrorReason,
};
use reqwest::StatusCode;
use serde::de::DeserializeOwned;

use crate::errors::{ClientError, ClientResult};

pub struct ServerResponse {
    http_response: reqwest::Response,
}

impl ServerResponse {
    pub(crate) fn from_reqwest_response(response: reqwest::Response) -> Self {
        Self {
            http_response: response,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn into_reqwest_response(self) -> reqwest::Response {
        self.http_response
    }

    /*
    #[deprecated]
    pub(crate) async fn prefetch_body(self) -> ClientResult<PrefetchedServerResponse> {
        let response_status_code = self.http_response.status();
        let response_headers = self.http_response.headers().to_owned();

        let response_body_bytes = self
            .http_response
            .bytes()
            .await
            .map_err(|error| ClientError::RequestExecutionError { error })?;


        Ok(PrefetchedServerResponse {
            response_status_code,
            response_headers,
            response_body_bytes,
            cached_error_reason: None,
        })
    } */

    pub(crate) fn status(&self) -> StatusCode {
        self.http_response.status()
    }

    pub(crate) async fn json<V>(self) -> ClientResult<V>
    where
        V: DeserializeOwned,
    {
        let body_data = self
            .http_response
            .bytes()
            .await
            .map_err(|error| ClientError::RequestExecutionError { error })?;

        serde_json::from_slice(&body_data)
            .map_err(|error| ClientError::ResponseJsonBodyError { error })
    }

    pub(crate) async fn error_reason(self) -> ClientResult<ErrorReason> {
        let response_with_error_reason = self.json::<ResponseWithErrorReason>().await?;

        Ok(response_with_error_reason.reason)
    }

    pub(crate) async fn category_error_reason(self) -> ClientResult<CategoryErrorReason> {
        let response_status = self.status();
        let error_reason = self.error_reason().await?;

        let ErrorReason::Category(category_error_reason) = error_reason else {
            return Err(ClientError::unexpected_error_reason(
                error_reason,
                response_status,
            ));
        };

        Ok(category_error_reason)
    }

    pub(crate) async fn word_error_reason(self) -> ClientResult<WordErrorReason> {
        let response_status = self.status();
        let error_reason = self.error_reason().await?;

        let ErrorReason::Word(word_error_reason) = error_reason else {
            return Err(ClientError::unexpected_error_reason(
                error_reason,
                response_status,
            ));
        };

        Ok(word_error_reason)
    }
}


/*
#[deprecated]
pub struct PrefetchedServerResponse {
    response_status_code: StatusCode,
    response_headers: HeaderMap,
    response_body_bytes: bytes::Bytes,
    cached_error_reason: Option<ErrorReason>,
}

impl PrefetchedServerResponse {
    pub(crate) fn json<V>(&self) -> ClientResult<V>
    where
        V: DeserializeOwned,
    {
        serde_json::from_slice(&self.response_body_bytes)
            .map_err(|error| ClientError::ResponseJsonBodyError { error })
    }

    pub(crate) fn json_error_reason(&mut self) -> ClientResult<ErrorReason> {
        if let Some(cached_error_reason) = &self.cached_error_reason {
            return Ok(cached_error_reason.to_owned());
        }


        let response_with_error_reason = self.json::<ResponseWithErrorReason>()?;
        self.cached_error_reason = Some(response_with_error_reason.reason.clone());

        Ok(response_with_error_reason.reason)
    }

    pub(crate) fn json_category_error_reason(&mut self) -> ClientResult<CategoryErrorReason> {
        let error_reason = self.json_error_reason()?;

        let ErrorReason::Category(category_error_reason) = error_reason else {
            return Err(ClientError::unexpected_error_reason(
                error_reason,
                self.response_status_code,
            ));
        };

        Ok(category_error_reason)
    }
}
 */
