pub mod endpoints;

use std::{
    env,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use kolomoni_api_client::{
    authentication::ClientAuthentication,
    client::{
        AuthenticatedKolomoniClient,
        UnauthenticatedClientOptions,
        UnauthenticatedKolomoniClient,
        UnrestrictedUnauthenticatedKolomoniClient,
    },
    server::KolomoniApiServer,
};

use crate::client::endpoints::{AssertableHealthEndpoints, AssertableTestingEndpoints};


pub const TEST_USER_AGENT: &str = concat!("kolomoni-e2e-test/v", env!("CARGO_PKG_VERSION"));

const TEST_SERVER_BASE_URL_ENVIRONMENT_VAR_NAME: &str = "KOLOMONI_TEST_SERVER_URL";


pub struct UnauthanticatedTestClient {
    client: UnauthenticatedKolomoniClient,
}

impl UnauthanticatedTestClient {
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

    /// Initializes a new unauthenticated kolomoni client based on the base URL
    /// provided in the `KOLOMONI_TEST_SERVER_URL` environment variable (e.g. "http://127.0.0.1:8866").
    ///
    /// The server is then verified to have been built with test flags enabled,
    /// and subsequently a full server reset is performed.
    pub async fn new_with_fresh_server() -> Self {
        let test_server_base_url = env::var(TEST_SERVER_BASE_URL_ENVIRONMENT_VAR_NAME)
            .unwrap_or_else(|error| {
                panic!(
                    "failed to initialize fresh test server: environment variable {} error: {:?}",
                    TEST_SERVER_BASE_URL_ENVIRONMENT_VAR_NAME, error
                )
            });

        let test_server = KolomoniApiServer::new_from_server_url_and_base_api_path(
            &test_server_base_url,
            "/api/v1/",
        )
        .unwrap_or_else(|error| {
            panic!(
                "failed to initialize fresh test server: invalid base url: {}",
                error
            )
        });

        let client = UnauthanticatedTestClient::new(test_server);

        client.testing().assert_testing_is_enabled_on_server().await;
        client.testing().perform_full_reset().await;

        client
    }

    pub fn with_authentication(
        &self,
        authentication: ClientAuthentication,
    ) -> AuthenticatedTestClient {
        AuthenticatedTestClient {
            client: self.client.with_authentication(authentication),
        }
    }

    pub fn to_unrestricted_test_client(&self) -> UnrestrictedUnauthenticatedTestClient {
        UnrestrictedUnauthenticatedTestClient::new(self.client.clone().into_unrestricted_client())
    }

    pub fn into_unrestricted_test_client(self) -> UnrestrictedUnauthenticatedTestClient {
        UnrestrictedUnauthenticatedTestClient::new(self.client.into_unrestricted_client())
    }
}

impl UnauthanticatedTestClient {
    pub fn testing<'c>(&'c self) -> AssertableTestingEndpoints<'c, UnauthenticatedKolomoniClient> {
        AssertableTestingEndpoints::new(&self.client)
    }

    pub fn assertable_health<'c>(
        &'c self,
    ) -> AssertableHealthEndpoints<'c, UnauthenticatedKolomoniClient> {
        AssertableHealthEndpoints::new(self.client.health())
    }
}

impl Deref for UnauthanticatedTestClient {
    type Target = UnauthenticatedKolomoniClient;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl DerefMut for UnauthanticatedTestClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}

impl AsRef<UnauthenticatedKolomoniClient> for UnauthanticatedTestClient {
    fn as_ref(&self) -> &UnauthenticatedKolomoniClient {
        &self.client
    }
}

impl AsMut<UnauthenticatedKolomoniClient> for UnauthanticatedTestClient {
    fn as_mut(&mut self) -> &mut UnauthenticatedKolomoniClient {
        &mut self.client
    }
}


pub struct UnrestrictedUnauthenticatedTestClient {
    client: UnrestrictedUnauthenticatedKolomoniClient,
}

impl UnrestrictedUnauthenticatedTestClient {
    pub fn new(client: UnrestrictedUnauthenticatedKolomoniClient) -> Self {
        Self { client }
    }
}

impl UnrestrictedUnauthenticatedTestClient {
    pub fn testing<'c>(
        &'c self,
    ) -> AssertableTestingEndpoints<'c, UnrestrictedUnauthenticatedKolomoniClient> {
        AssertableTestingEndpoints::new(&self.client)
    }

    pub fn assertable_health<'c>(
        &'c self,
    ) -> AssertableHealthEndpoints<'c, UnrestrictedUnauthenticatedKolomoniClient> {
        AssertableHealthEndpoints::new(self.client.health())
    }
}

impl Deref for UnrestrictedUnauthenticatedTestClient {
    type Target = UnrestrictedUnauthenticatedKolomoniClient;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl DerefMut for UnrestrictedUnauthenticatedTestClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}

impl AsRef<UnrestrictedUnauthenticatedKolomoniClient> for UnrestrictedUnauthenticatedTestClient {
    fn as_ref(&self) -> &UnrestrictedUnauthenticatedKolomoniClient {
        &self.client
    }
}

impl AsMut<UnrestrictedUnauthenticatedKolomoniClient> for UnrestrictedUnauthenticatedTestClient {
    fn as_mut(&mut self) -> &mut UnrestrictedUnauthenticatedKolomoniClient {
        &mut self.client
    }
}



pub struct AuthenticatedTestClient {
    client: AuthenticatedKolomoniClient,
}

impl AuthenticatedTestClient {
    pub fn testing<'c>(&'c self) -> AssertableTestingEndpoints<'c, AuthenticatedKolomoniClient> {
        AssertableTestingEndpoints::new(&self.client)
    }

    pub fn assertable_health<'c>(
        &'c self,
    ) -> AssertableHealthEndpoints<'c, AuthenticatedKolomoniClient> {
        AssertableHealthEndpoints::new(self.client.health())
    }
}

impl Deref for AuthenticatedTestClient {
    type Target = AuthenticatedKolomoniClient;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl DerefMut for AuthenticatedTestClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}

impl AsRef<AuthenticatedKolomoniClient> for AuthenticatedTestClient {
    fn as_ref(&self) -> &AuthenticatedKolomoniClient {
        &self.client
    }
}

impl AsMut<AuthenticatedKolomoniClient> for AuthenticatedTestClient {
    fn as_mut(&mut self) -> &mut AuthenticatedKolomoniClient {
        &mut self.client
    }
}
