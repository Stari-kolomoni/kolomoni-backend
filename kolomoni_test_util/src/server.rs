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

use crate::{macros::assert_request_ok, response::AssertableServerResponse};


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
        let response = assert_request_ok!(
            self.client
                .get_request_builder()
                .raw_endpoint_url("/testing/enabled")
                .send()
                .await,
            "failed to check whether the server has been compiled with the testing feature flag"
        );

        if response.status() != StatusCode::OK {
            panic!("expected the server to have the testing feature flag enabled");
        }
    }

    pub async fn perform_full_reset(&self) {
        self.assert_testing_is_enabled_on_server().await;

        let response = assert_request_ok!(
            self.client
                .post_request_builder()
                .raw_endpoint_url("/testing/state/reset")
                .send()
                .await,
            "failed to execute request to perform full backend reset"
        );

        if response.status() != StatusCode::OK {
            panic!(
                "failed to perform full backend reset: {}",
                response.format_with_debug_info()
            );
        }
    }

    pub async fn give_user_administrator_role(&self, user_id: UserId) {
        self.assert_testing_is_enabled_on_server().await;

        let response = assert_request_ok!(
            self.client
                .post_request_builder()
                .raw_endpoint_url("/testing/user/give-administrator-role")
                .json(&GiveAdministratorRoleRequest {
                    user_id: user_id.into_uuid(),
                })
                .send()
                .await,
            format!(
                "failed to execute request to give user {:?} the administrator role",
                user_id
            )
        );


        if response.status() != StatusCode::OK {
            panic!(
                "failed to give user {:?} the administrator role: {}",
                user_id,
                response.format_with_debug_info()
            );
        }
    }

    pub async fn reset_user_roles_to_default(&self, user_id: UserId) {
        self.assert_testing_is_enabled_on_server().await;

        let response = assert_request_ok!(
            self.client
                .post_request_builder()
                .raw_endpoint_url("/testing/user/reset-roles-to-default")
                .json(&ResetUserRolesRequest {
                    user_id: user_id.into_uuid(),
                })
                .send()
                .await,
            format!(
                "failed to execute request to reset roles for user {:?} to default",
                user_id
            )
        );

        if response.status() != StatusCode::OK {
            panic!(
                "failed to reset user {:?}'s roles to default: {}",
                user_id,
                response.format_with_debug_info()
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
        let ping_result = assert_request_ok!(
            self.client.health().ping().await,
            "failed to ping server health endpoint"
        );

        assert!(ping_result, "failed to ping server");
    }
}



pub trait TestServerClient {
    type Testing<'c>
    where
        Self: 'c;

    type AssertableHealth<'c>
    where
        Self: 'c;

    fn testing(&self) -> Self::Testing<'_>;

    fn assertable_health(&self) -> Self::AssertableHealth<'_>;
}




pub struct UnauthanticatedTestServerClient {
    client: kolomoni_api_client::UnauthenticatedClient,
}

impl UnauthanticatedTestServerClient {
    pub fn new<S>(server: S) -> Self
    where
        S: Into<ApiServer>,
    {
        let client = kolomoni_api_client::UnauthenticatedClient::new_with_options(
            Arc::new(server.into()),
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
    ) -> AuthenticatedTestServerClient {
        AuthenticatedTestServerClient {
            client: self.client.with_authentication(authentication),
        }
    }
}

impl TestServerClient for UnauthanticatedTestServerClient {
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

impl Deref for UnauthanticatedTestServerClient {
    type Target = kolomoni_api_client::UnauthenticatedClient;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}



pub struct AuthenticatedTestServerClient {
    client: kolomoni_api_client::AuthenticatedClient,
}

impl TestServerClient for AuthenticatedTestServerClient {
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

impl Deref for AuthenticatedTestServerClient {
    type Target = kolomoni_api_client::AuthenticatedClient;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}
