use std::fmt::Debug;

use actix_http::{
    header::{HeaderName, HeaderValue},
    Method,
    StatusCode,
};
use actix_web::http;
use bytes::Bytes;
use kolomoni::api::v1::ping::PingResponse;
use reqwest::{header::HeaderMap, Client, ClientBuilder, RequestBuilder, Response};
use serde::{Deserialize, Serialize};


pub const TEST_USER_AGENT: &str = concat!("kolomoni-e2e-test/", env!("CARGO_PKG_VERSION"));

pub struct TestServer {
    base_api_url: String,

    client: Client,
}

impl TestServer {
    pub fn new(base_api_url: String) -> Self {
        let client = ClientBuilder::new()
            .user_agent(TEST_USER_AGENT)
            .build()
            .expect("failed to set up reqwest client");

        Self {
            client,
            base_api_url,
        }
    }

    pub async fn reset_server(&self) {
        let response = self
            .request(Method::POST, "/testing/full-reset")
            .send()
            .await;

        response.assert_status_equals(StatusCode::OK);
    }

    pub fn request<U>(&self, method: http::Method, endpoint: U) -> TestRequestBuilder
    where
        U: AsRef<str>,
    {
        let request_builder = self.client.request(
            method,
            format!("{}{}", self.base_api_url, endpoint.as_ref()),
        );

        TestRequestBuilder { request_builder }
    }
}


pub struct TestRequestBuilder {
    request_builder: RequestBuilder,
}

impl TestRequestBuilder {
    pub fn with_authentication_token(mut self, token: String) -> Self {
        self.request_builder = self.request_builder.bearer_auth(token);
        self
    }

    pub fn with_json_body<V>(mut self, value: V) -> Self
    where
        V: Serialize,
    {
        let serialized_body = serde_json::to_vec(&value).expect("failed to serialize value to JSON");

        self.request_builder = self.request_builder.body(serialized_body);
        self
    }

    pub async fn send(self) -> TestResponseV2 {
        let response = self
            .request_builder
            .send()
            .await
            .expect("failed to perform HTTP request");

        TestResponseV2::from_reqwest_response(response).await
    }
}


pub struct TestResponseV2 {
    status: StatusCode,
    headers: HeaderMap,
    body_bytes: Bytes,
}

impl TestResponseV2 {
    async fn from_reqwest_response(response: Response) -> Self {
        Self {
            status: response.status(),
            headers: response.headers().to_owned(),
            body_bytes: response
                .bytes()
                .await
                .expect("failed to extract body from response"),
        }
    }

    pub fn assert_status_equals(&self, status_code: StatusCode) {
        assert_eq!(self.status, status_code);
    }

    pub fn assert_header_exists<N>(&self, header_name: N)
    where
        N: Into<HeaderName>,
    {
        let header_name: HeaderName = header_name.into();

        self.headers.get(&header_name).unwrap_or_else(|| {
            panic!(
                "header {} does not exist on response",
                header_name.as_str()
            )
        });
    }

    pub fn assert_header_matches_value<N, V>(&self, header_name: N, header_value: V)
    where
        N: Into<HeaderName>,
        V: Into<HeaderValue>,
    {
        let header_name: HeaderName = header_name.into();
        let expected_header_value: HeaderValue = header_value.into();

        let actual_header_value = self.headers.get(&header_name).unwrap_or_else(|| {
            panic!(
                "header {} does not exist on response",
                header_name.as_str()
            )
        });

        assert_eq!(expected_header_value, actual_header_value);
    }

    pub fn json_body<'de, D>(&'de self) -> D
    where
        D: Deserialize<'de>,
    {
        serde_json::from_slice::<D>(&self.body_bytes).expect("failed to deserialize body as JSON")
    }

    pub fn assert_json_body_matches<'de, D>(&'de self, expected_content: D)
    where
        D: Deserialize<'de> + PartialEq + Eq + Debug,
    {
        let data = self.json_body::<D>();

        assert_eq!(data, expected_content);
    }
}


pub async fn prepare_test_server_instance() -> TestServer {
    const TEST_API_SERVER_ENV_VAR_NAME: &str = "TEST_API_SERVER_URL";

    let test_server_url = std::env::var(TEST_API_SERVER_ENV_VAR_NAME).unwrap_or_else(|_| {
        panic!(
            "Unexpected test environment! Expected a {} environment variable, found none (or invalid unicode).",
            TEST_API_SERVER_ENV_VAR_NAME
        )
    });

    let server = TestServer::new(test_server_url);
    server.reset_server().await;

    server
}


#[tokio::test]
async fn server_can_be_pinged() {
    let server = prepare_test_server_instance().await;

    let response = server.request(Method::GET, "/api/v1/ping").send().await;

    response.assert_status_equals(StatusCode::OK);
    response.assert_json_body_matches(PingResponse { ok: true });
}
