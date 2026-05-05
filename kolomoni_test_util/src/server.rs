use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use kolomoni_api_client::{
    api::health::{HealthAnonymousEndpoints, HealthApi},
    authentication::ClientAuthentication,
    client::{
        AuthenticatedKolomoniClient,
        KolomoniHttpClient,
        UnauthenticatedClientOptions,
        UnauthenticatedKolomoniClient,
    },
    request::ToRequestBuilder,
    server::KolomoniApiServer,
};
use kolomoni_core::{
    api_models::{GiveAdministratorRoleRequest, ResetUserRolesRequest},
    ids::UserId,
};
use reqwest::StatusCode;


pub const TEST_USER_AGENT: &str = concat!("kolomoni-e2e-test/v", env!("CARGO_PKG_VERSION"));



pub struct TestingEndpoints<'s, C>
where
    C: KolomoniHttpClient,
{
    client: &'s C,
}

impl<'s, C> TestingEndpoints<'s, C>
where
    C: KolomoniHttpClient,
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
            .get()
            .endpoint_url_without_base_path("/testing/enabled")
            .build_request()
            .send()
            .await
            .expect(
                "failed to check whether the server has been compiled with the testing feature flag",
            );

        if response.status() != StatusCode::OK {
            panic!("expected the server to have the testing feature flag enabled");
        }
    }

    pub async fn perform_full_reset(&self) {
        self.assert_testing_is_enabled_on_server().await;

        let response = self
            .client
            .post()
            .endpoint_url_without_base_path("/testing/state/reset")
            .build_request()
            .send()
            .await
            .expect("failed to execute request to perform full backend reset");

        if response.status() != StatusCode::OK {
            panic!(
                "failed to perform full backend reset: got status code {}",
                response.status()
            );
        }
    }

    pub async fn give_user_administrator_role(&self, user_id: UserId) {
        self.assert_testing_is_enabled_on_server().await;

        let response = self
            .client
            .post()
            .endpoint_url_without_base_path("/testing/user/give-administrator-role")
            .json(&GiveAdministratorRoleRequest {
                user_id: user_id.into_uuid(),
            })
            .build_request()
            .send()
            .await
            .unwrap_or_else(|error| {
                panic!(
                    "failed to execute request to give user {:?} the administrator role: {}",
                    user_id, error
                )
            });


        if response.status() != StatusCode::OK {
            panic!(
                "failed to give user {:?} the administrator role: got status code {}",
                user_id,
                response.status()
            );
        }
    }

    pub async fn reset_user_roles_to_default(&self, user_id: UserId) {
        self.assert_testing_is_enabled_on_server().await;

        let response = self
            .client
            .post()
            .endpoint_url_without_base_path("/testing/user/reset-roles-to-default")
            .json(&ResetUserRolesRequest {
                user_id: user_id.into_uuid(),
            })
            .build_request()
            .send()
            .await
            .unwrap_or_else(|error| {
                panic!(
                    "failed to execute request to reset roles for user {:?} to default: {}",
                    user_id, error
                )
            });

        if response.status() != StatusCode::OK {
            panic!(
                "failed to reset user {:?}'s roles to default: got status code {}",
                user_id,
                response.status()
            );
        }
    }
}


pub struct AssertableHealthEndpoints<'c, C>
where
    C: KolomoniHttpClient,
{
    inner: HealthApi<'c, C>,
}

impl<'c, C> AssertableHealthEndpoints<'c, C>
where
    C: KolomoniHttpClient,
{
    #[inline]
    fn new(inner: HealthApi<'c, C>) -> Self {
        Self { inner }
    }

    pub async fn assert_server_ping_is_ok(&self) {
        let ping_result = self
            .inner
            .ping()
            .send()
            .await
            .expect("failed to ping server");

        assert!(ping_result, "server is not healthy");
    }
}



#[deprecated]
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
    client: UnauthenticatedKolomoniClient,
}

impl UnauthanticatedTestServerClient {
    pub fn new<S>(server: S) -> Self
    where
        S: Into<KolomoniApiServer>,
    {
        let client = UnauthenticatedKolomoniClient::new_with_options(
            Arc::new(server.into()),
            UnauthenticatedClientOptions::default(),
        )
        .expect("failed to initialize API client with provided server");

        Self { client }
    }

    pub fn with_authentication(
        &self,
        authentication: ClientAuthentication,
    ) -> AuthenticatedTestServerClient {
        AuthenticatedTestServerClient {
            client: self.client.with_authentication(authentication),
        }
    }
}

impl UnauthanticatedTestServerClient {
    pub fn testing<'c>(&'c self) -> TestingEndpoints<'c, UnauthenticatedKolomoniClient> {
        TestingEndpoints::new(&self.client)
    }

    pub fn assertable_health<'c>(
        &'c self,
    ) -> AssertableHealthEndpoints<'c, UnauthenticatedKolomoniClient> {
        AssertableHealthEndpoints::new(self.client.health())
    }
}

impl Deref for UnauthanticatedTestServerClient {
    type Target = UnauthenticatedKolomoniClient;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl DerefMut for UnauthanticatedTestServerClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}

impl AsRef<UnauthenticatedKolomoniClient> for UnauthanticatedTestServerClient {
    fn as_ref(&self) -> &UnauthenticatedKolomoniClient {
        &self.client
    }
}

impl AsMut<UnauthenticatedKolomoniClient> for UnauthanticatedTestServerClient {
    fn as_mut(&mut self) -> &mut UnauthenticatedKolomoniClient {
        &mut self.client
    }
}



pub struct AuthenticatedTestServerClient {
    client: AuthenticatedKolomoniClient,
}

impl AuthenticatedTestServerClient {
    pub fn testing<'c>(&'c self) -> TestingEndpoints<'c, AuthenticatedKolomoniClient> {
        TestingEndpoints::new(&self.client)
    }

    pub fn assertable_health<'c>(
        &'c self,
    ) -> AssertableHealthEndpoints<'c, AuthenticatedKolomoniClient> {
        AssertableHealthEndpoints::new(self.client.health())
    }
}

impl Deref for AuthenticatedTestServerClient {
    type Target = AuthenticatedKolomoniClient;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl DerefMut for AuthenticatedTestServerClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}

impl AsRef<AuthenticatedKolomoniClient> for AuthenticatedTestServerClient {
    fn as_ref(&self) -> &AuthenticatedKolomoniClient {
        &self.client
    }
}

impl AsMut<AuthenticatedKolomoniClient> for AuthenticatedTestServerClient {
    fn as_mut(&mut self) -> &mut AuthenticatedKolomoniClient {
        &mut self.client
    }
}
