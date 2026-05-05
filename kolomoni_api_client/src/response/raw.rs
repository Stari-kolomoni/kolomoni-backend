use std::future::Future;

use bytes::Bytes;
use kolomoni_core::api_models::{
    CategoryErrorReason,
    ErrorReason,
    LoginErrorReason,
    ResponseWithErrorReason,
    TranslationsErrorReason,
    UsersErrorReason,
    WordErrorReason,
};
use reqwest::{header::HeaderMap, StatusCode};
use serde::de::DeserializeOwned;

use crate::{
    client::errors::{RequestResult, RequestError},
    response::{ResponseValueError, TypedResponse},
};

pub struct RawResponse {
    http_response: reqwest::Response,
}

impl RawResponse {
    pub(crate) fn new(response: reqwest::Response) -> Self {
        Self {
            http_response: response,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn into_reqwest_response(self) -> reqwest::Response {
        self.http_response
    }

    pub fn status(&self) -> StatusCode {
        self.http_response.status()
    }

    pub fn headers(&self) -> &HeaderMap {
        self.http_response.headers()
    }

    pub async fn body_bytes(self) -> RequestResult<Bytes> {
        self.http_response
            .bytes()
            .await
            .map_err(|error| RequestError::RequestExecutionError { error })
    }

    pub async fn into_json_body<V>(self) -> RequestResult<V>
    where
        V: DeserializeOwned,
    {
        let status = self.status();
        let body_data = self.body_bytes().await?;

        let parsed_json_body = serde_json::from_slice::<V>(&body_data);

        match parsed_json_body {
            Ok(value) => Ok(value),
            Err(parsing_error) => {
                let maybe_error_reason =
                    serde_json::from_slice::<ResponseWithErrorReason>(&body_data);

                match maybe_error_reason {
                    Ok(error_reason_data) => Err(RequestError::unexpected_error_reason(
                        error_reason_data.reason,
                        status,
                    )),
                    Err(_) => {
                        // The body contains neither V nor ResponseWithErrorReason,
                        // which means the response is something else entirely we did not expect,
                        // basically a deserialization error.
                        Err(RequestError::ResponseJsonBodyError {
                            error: parsing_error,
                        })
                    }
                }
            }
        }
    }

    pub fn into_typed_response<V, E, F, Fut>(self, parser_closure: F) -> TypedResponse<V, E>
    where
        V: DeserializeOwned,
        E: ResponseValueError,
        F: FnOnce(RawResponse) -> Fut + 'static,
        Fut: Future<Output = Result<V, E>> + Send + 'static,
    {
        TypedResponse::new(Ok(self), parser_closure)
    }

    // pub async fn into_typed_parsed_response<V>(self) -> ServerResponse<V>
    // where
    //     V: DeserializeOwned,
    // {
    //     let data = self.into_json_body::<V>().await;
    //
    //     ServerResponse::<V>::from_typed_data_result(data)
    // }

    /// Consumes the response and parses the body as an [`ErrorReason`].
    /// Because deserialization might fail, this returns a [`ClientResult`].
    pub async fn error_reason(self) -> RequestResult<ErrorReason> {
        let body_data = self.body_bytes().await?;

        let parsed_json_body = serde_json::from_slice::<ResponseWithErrorReason>(&body_data)
            .map_err(|error| RequestError::ResponseJsonBodyError { error })?;

        Ok(parsed_json_body.reason)
    }

    /// Consumes the response and parses the body as a specific error reason: a [`LoginErrorReason`].
    ///
    /// Because deserialization might fail, this returns a [`ClientResult`]. Also returns an error
    /// when the response has a valid [`ErrorReason`], but it's not a [`LoginErrorReason`].
    ///
    /// See also: [`Self::error_reason`]
    pub async fn login_error_reason(self) -> RequestResult<LoginErrorReason> {
        let response_status = self.status();
        let error_reason = self.error_reason().await?;

        let ErrorReason::Login(login_error_reason) = error_reason else {
            return Err(RequestError::unexpected_error_reason(
                error_reason,
                response_status,
            ));
        };

        Ok(login_error_reason)
    }

    /// Consumes the response and parses the body as a specific error reason: a [`CategoryErrorReason`].
    ///
    /// Because deserialization might fail, this returns a [`ClientResult`]. Also returns an error
    /// when the response has a valid [`ErrorReason`], but it's not a [`CategoryErrorReason`].
    ///
    /// See also: [`Self::error_reason`]
    pub async fn category_error_reason(self) -> RequestResult<CategoryErrorReason> {
        let response_status = self.status();
        let error_reason = self.error_reason().await?;

        let ErrorReason::Category(category_error_reason) = error_reason else {
            return Err(RequestError::unexpected_error_reason(
                error_reason,
                response_status,
            ));
        };

        Ok(category_error_reason)
    }

    /// Consumes the response and parses the body as a specific error reason: a [`WordErrorReason`].
    ///
    /// Because deserialization might fail, this returns a [`ClientResult`]. Also returns an error
    /// when the response has a valid [`ErrorReason`], but it's not a [`WordErrorReason`].
    ///
    /// See also: [`Self::error_reason`]
    pub async fn word_error_reason(self) -> RequestResult<WordErrorReason> {
        let response_status = self.status();
        let error_reason = self.error_reason().await?;

        let ErrorReason::Word(word_error_reason) = error_reason else {
            return Err(RequestError::unexpected_error_reason(
                error_reason,
                response_status,
            ));
        };

        Ok(word_error_reason)
    }

    /// Consumes the response and parses the body as a specific error reason: a [`TranslationsErrorReason`].
    ///
    /// Because deserialization might fail, this returns a [`ClientResult`]. Also returns an error
    /// when the response has a valid [`ErrorReason`], but it's not a [`TranslationsErrorReason`].
    ///
    /// See also: [`Self::error_reason`]
    pub async fn translations_error_reason(self) -> RequestResult<TranslationsErrorReason> {
        let response_status = self.status();
        let error_reason = self.error_reason().await?;

        let ErrorReason::Translations(translations_error_reason) = error_reason else {
            return Err(RequestError::unexpected_error_reason(
                error_reason,
                response_status,
            ));
        };

        Ok(translations_error_reason)
    }

    /// Consumes the response and parses the body as a specific error reason: a [`UsersErrorReason`].
    ///
    /// Because deserialization might fail, this returns a [`ClientResult`]. Also returns an error
    /// when the response has a valid [`ErrorReason`], but it's not a [`UsersErrorReason`].
    ///
    /// See also: [`Self::error_reason`]
    pub async fn users_error_reason(self) -> RequestResult<UsersErrorReason> {
        let response_status = self.status();
        let error_reason = self.error_reason().await?;

        let ErrorReason::Users(users_error_reason) = error_reason else {
            return Err(RequestError::unexpected_error_reason(
                error_reason,
                response_status,
            ));
        };

        Ok(users_error_reason)
    }
}
