use ::http::{HeaderMap, HeaderName};
use actix_http::{header::HeaderValue, Method, StatusCode};
use actix_web::http;
use kolomoni::testing::{GiveFullUserPermissionsRequest, ResetUserRolesRequest};
use reqwest::{header, Client, ClientBuilder, RequestBuilder};
use serde::Serialize;

use crate::TestResponse;

pub const TEST_USER_AGENT: &str = concat!("kolomoni-e2e-test/", env!("CARGO_PKG_VERSION"));

pub struct TestServer {
    base_api_url: String,

    client: Client,
}

impl TestServer {
    pub fn new(base_api_url: String) -> Self {
        let var_name = ClientBuilder::new();
        let client = var_name
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

    pub async fn give_full_permissions_to_user(&self, user_id: i32) {
        let response = self
            .request(
                Method::POST,
                "/testing/give-user-full-permissions",
            )
            .with_json_body(GiveFullUserPermissionsRequest { user_id })
            .send()
            .await;

        response.assert_status_equals(StatusCode::OK);
    }

    pub async fn reset_user_permissions_to_normal(&self, user_id: i32) {
        let response = self
            .request(
                Method::POST,
                "/testing/reset-user-roles-to-normal",
            )
            .with_json_body(ResetUserRolesRequest { user_id })
            .send()
            .await;

        response.assert_status_equals(StatusCode::OK);
    }

    pub fn request<U>(&self, method: http::Method, endpoint: U) -> TestRequestBuilder
    where
        U: AsRef<str>,
    {
        TestRequestBuilder::new(
            &self.client,
            method,
            format!("{}{}", self.base_api_url, endpoint.as_ref()),
        )
    }
}


#[derive(Debug, PartialEq, Eq)]
pub struct TestRequestDebugInfo {
    url: String,
    method: Method,
    headers: HeaderMap,
    body: Option<String>,
}


pub struct TestRequestBuilder {
    debug_info: TestRequestDebugInfo,
    request_builder: RequestBuilder,
}

impl TestRequestBuilder {
    pub fn new(client: &Client, method: Method, url: String) -> Self {
        Self {
            debug_info: TestRequestDebugInfo {
                url: url.clone(),
                method: method.clone(),
                headers: HeaderMap::new(),
                body: None,
            },
            request_builder: client.request(method, url),
        }
    }

    pub fn with_header(mut self, header_name: HeaderName, header_value: HeaderValue) -> Self {
        self.debug_info
            .headers
            .append(header_name.clone(), header_value.clone());

        self.request_builder = self.request_builder.header(header_name, header_value);
        self
    }

    pub fn with_access_token<S>(mut self, token: S) -> Self
    where
        S: Into<String>,
    {
        let token: String = token.into();

        self.debug_info.headers.append(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        );

        self.request_builder = self.request_builder.bearer_auth(token);
        self
    }

    pub fn with_json_body<V>(mut self, value: V) -> Self
    where
        V: Serialize,
    {
        let serialized_body = serde_json::to_vec(&value).expect("failed to serialize value to JSON");

        self.debug_info.body = Some(String::from_utf8(serialized_body.clone()).unwrap());

        self.request_builder = self.request_builder.body(serialized_body);
        self.request_builder = self.request_builder.header(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        self
    }

    pub async fn send(self) -> TestResponse {
        let response = self
            .request_builder
            .send()
            .await
            .expect("failed to perform HTTP request");

        TestResponse::new(self.debug_info, response).await
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
