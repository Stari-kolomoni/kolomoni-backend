use kolomoni_api_client::{
    api::health::{HealthAnonymousEndpoints, HealthApi},
    client::KolomoniHttpClient,
    request::ToRequestBuilder,
};
use kolomoni_core::{
    api_models::{GiveAdministratorRoleRequest, ResetUserRolesRequest},
    ids::UserId,
};
use reqwest::StatusCode;

pub struct AssertableTestingEndpoints<'s, C>
where
    C: KolomoniHttpClient,
{
    client: &'s C,
}

impl<'s, C> AssertableTestingEndpoints<'s, C>
where
    C: KolomoniHttpClient,
{
    #[inline]
    pub(super) fn new(client: &'s C) -> Self {
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
    pub(super) fn new(inner: HealthApi<'c, C>) -> Self {
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
