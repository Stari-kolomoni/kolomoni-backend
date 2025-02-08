use std::{borrow::Cow, ops::Deref, sync::Arc};

use kolomoni_api_client::{
    api::health::SharedHealthEndpoints,
    authentication::ServerAuthentication,
    request::ApiClientRequestBuild,
    ApiClient,
    ApiServer,
    ClientOptions,
    SharedApiClientEndpointGroups,
};
use kolomoni_core::{
    api_models::{GiveAdministratorRoleRequest, ResetUserRolesRequest},
    ids::UserId,
};
use reqwest::StatusCode;

pub const TEST_USER_AGENT: &str = concat!("kolomoni-e2e-test/v", env!("CARGO_PKG_VERSION"));



pub struct TestingEndpoints<'s, C>
where
    C: kolomoni_api_client::ApiClient,
{
    client: &'s C,
}

impl<'s, C> TestingEndpoints<'s, C>
where
    C: kolomoni_api_client::ApiClient,
{
    #[inline]
    fn new(client: &'s C) -> Self {
        Self { client }
    }

    /// Asserts the server binary we are testing on has been compiled
    /// with the `e2e-testing` feature flag, exposing the required
    /// additional endpoints we use while testing.
    pub async fn assert_testing_is_enabled_on_server(&self) {
        let response = self
            .client
            .get_request_builder()
            .endpoint_url("/testing/enabled")
            .send()
            .await
            .unwrap();

        if response.status() != StatusCode::OK {
            panic!("expected the server to have the testing feature flag enabled");
        }
    }

    pub async fn perform_full_reset(&self) {
        self.assert_testing_is_enabled_on_server().await;

        let response = self
            .client
            .post_request_builder()
            .endpoint_url("/testing/state/reset")
            .send()
            .await
            .unwrap();

        if response.status() != StatusCode::OK {
            panic!("failed to perform full backend reset")
        }
    }

    pub async fn give_user_administrator_role(&self, user_id: UserId) {
        self.assert_testing_is_enabled_on_server().await;

        let response = self
            .client
            .post_request_builder()
            .endpoint_url("/testing/user/give-administrator-role")
            .json(&GiveAdministratorRoleRequest { user_id })
            .send()
            .await
            .unwrap();


        if response.status() != StatusCode::OK {
            panic!(
                "failed to give user {:?} administrator role",
                user_id
            );
        }
    }

    pub async fn reset_user_roles_to_default(&self, user_id: UserId) {
        self.assert_testing_is_enabled_on_server().await;

        let response = self
            .client
            .post_request_builder()
            .endpoint_url("/testing/user/reset-roles-to-default")
            .json(&ResetUserRolesRequest { user_id })
            .send()
            .await
            .unwrap();

        if response.status() != StatusCode::OK {
            panic!(
                "failed to reset user {:?}'s roles to default",
                user_id
            );
        }
    }
}


pub struct AssertableHealthEndpoints<'s, C>
where
    C: ApiClient + SharedApiClientEndpointGroups,
{
    client: &'s C,
}

impl<'s, C> AssertableHealthEndpoints<'s, C>
where
    C: ApiClient + SharedApiClientEndpointGroups,
{
    #[inline]
    fn new(client: &'s C) -> Self {
        Self { client }
    }

    pub async fn assert_server_can_be_pinged(&self) {
        let ping_result = self
            .client
            .health()
            .ping()
            .await
            .expect("failed to ping server health endpoint");

        assert!(ping_result);
    }
}



pub trait TestServer {
    type Testing<'c>
    where
        Self: 'c;

    type AssertableHealth<'c>
    where
        Self: 'c;

    fn testing(&self) -> Self::Testing<'_>;

    fn assertable_health(&self) -> Self::AssertableHealth<'_>;
}




pub struct UnauthanticatedTestServer {
    client: kolomoni_api_client::UnauthenticatedClient,
}

impl UnauthanticatedTestServer {
    pub fn new<S>(server: S) -> Self
    where
        S: Into<ApiServer>,
    {
        let client = kolomoni_api_client::UnauthenticatedClient::new_with_options(
            &Arc::new(server.into()),
            ClientOptions {
                user_agent: Cow::Borrowed(TEST_USER_AGENT),
            },
        )
        .expect("failed to initialize API client with provided server");

        Self { client }
    }

    pub fn with_authentication(
        &self,
        authentication: ServerAuthentication,
    ) -> AuthenticatedTestServer {
        AuthenticatedTestServer {
            client: self.client.with_authentication(authentication),
        }
    }
}

impl TestServer for UnauthanticatedTestServer {
    type Testing<'c> = TestingEndpoints<'c, kolomoni_api_client::UnauthenticatedClient>;

    type AssertableHealth<'c> =
        AssertableHealthEndpoints<'c, kolomoni_api_client::UnauthenticatedClient>;

    fn testing(&self) -> Self::Testing<'_> {
        TestingEndpoints::new(&self.client)
    }

    fn assertable_health(&self) -> Self::AssertableHealth<'_> {
        AssertableHealthEndpoints::new(&self.client)
    }
}

impl Deref for UnauthanticatedTestServer {
    type Target = kolomoni_api_client::UnauthenticatedClient;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}



// TODO need an upgrade method from TestServer
pub struct AuthenticatedTestServer {
    client: kolomoni_api_client::AuthenticatedClient,
}

impl TestServer for AuthenticatedTestServer {
    type Testing<'c> = TestingEndpoints<'c, kolomoni_api_client::AuthenticatedClient>;

    type AssertableHealth<'c> =
        AssertableHealthEndpoints<'c, kolomoni_api_client::AuthenticatedClient>;

    fn testing(&self) -> Self::Testing<'_> {
        TestingEndpoints::new(&self.client)
    }

    fn assertable_health(&self) -> Self::AssertableHealth<'_> {
        AssertableHealthEndpoints::new(&self.client)
    }
}

impl Deref for AuthenticatedTestServer {
    type Target = kolomoni_api_client::AuthenticatedClient;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}


/*
pub struct TestServer_OLD {
    client: kolomoni_api_client::Client,
}

impl TestServer_OLD {
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

    pub async fn reset_database(&self) {
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



pub async fn initialize_test_server() -> TestServer_OLD {
    const TEST_API_SERVER_ENV_VAR_NAME: &str = "TEST_API_SERVER_URL";

    let test_server_url = std::env::var(TEST_API_SERVER_ENV_VAR_NAME).unwrap_or_else(|_| {
        panic!(
            "Unexpected test environment! Expected a {} environment variable, found none (or invalid unicode).",
            TEST_API_SERVER_ENV_VAR_NAME
        )
    });

    let server = TestServer_OLD::new(test_server_url);
    server.reset_database().await;

    server
}
 */
