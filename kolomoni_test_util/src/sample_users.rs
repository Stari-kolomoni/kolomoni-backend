use chrono::Utc;
use kolomoni_api_client::{
    api::auth::{UserLoginInfo, UserRegistrationInfo},
    authentication::ServerTokenSet,
    SharedApiClientEndpointGroups,
    UnauthenticatedKolomoniClient,
};
use kolomoni_core::api_models::UserInfo;

/// A sample user intended for testing the backend.
/// Each user has an associated username, display name and password.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SampleUser {
    Janez,
    Meta,
    Kira,
}

impl SampleUser {
    pub fn username(&self) -> &'static str {
        match self {
            SampleUser::Janez => "janez",
            SampleUser::Meta => "meta",
            SampleUser::Kira => "kira",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            SampleUser::Janez => "Janez Jasnovidni",
            SampleUser::Meta => "Meta Meglenska",
            SampleUser::Kira => "Kira",
        }
    }

    pub fn password(&self) -> &'static str {
        match self {
            SampleUser::Janez => "janez",
            SampleUser::Meta => "meta",
            SampleUser::Kira => "kira",
        }
    }

    /// Registers the given sample user on the server,
    /// returning their fresh user information.
    pub async fn register(&self, client: &UnauthenticatedKolomoniClient) -> UserInfo {
        let before_registration = Utc::now();

        let newly_registered_user = client
            .authentication()
            .register_user(UserRegistrationInfo {
                username: self.username().to_owned(),
                display_name: self.display_name().to_owned(),
                password: self.password().to_owned(),
            })
            .await
            .expect("failed to create sample user account");

        let after_registration = Utc::now();

        assert_eq!(
            newly_registered_user.user.display_name,
            self.display_name()
        );
        assert_eq!(
            newly_registered_user.user.username,
            self.username()
        );

        assert!(newly_registered_user.user.joined_at >= before_registration);
        assert!(newly_registered_user.user.joined_at <= after_registration);

        assert_eq!(
            newly_registered_user.user.joined_at,
            newly_registered_user.user.last_active_at
        );
        assert_eq!(
            newly_registered_user.user.joined_at,
            newly_registered_user.user.last_modified_at
        );


        newly_registered_user.user
    }

    /// Logins the user and returns the access and refres token as [`ServerTokenSet`]
    /// (which can be turned into [`ServerAuthentication`], which can, in turn, be used to upgrade
    /// an unauthenticated client into an authenticated one).
    pub async fn login(&self, client: &UnauthenticatedKolomoniClient) -> ServerTokenSet {
        let tokens = client
            .authentication()
            .login_user(UserLoginInfo {
                username: self.username().to_owned(),
                password: self.password().to_owned(),
            })
            .await
            .expect("failed to perform sample user login");

        ServerTokenSet::new(tokens.access_token, tokens.refresh_token)
    }
}
