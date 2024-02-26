use http::{header, Method, StatusCode};
use kolomoni::api::v1::{
    login::{UserLoginRequest, UserLoginResponse},
    users::{
        registration::{UserRegistrationRequest, UserRegistrationResponse},
        UserInfoResponse,
        UserInformation,
    },
};

use crate::TestServer;


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

    pub fn into_registration_request_model(self) -> UserRegistrationRequest {
        UserRegistrationRequest {
            username: self.username().to_string(),
            password: self.password().to_string(),
            display_name: self.display_name().to_string(),
        }
    }

    pub fn into_login_request_model(self) -> UserLoginRequest {
        UserLoginRequest {
            username: self.username().to_string(),
            password: self.password().to_string(),
        }
    }

    /// Registers the given [`SampleUser`] on the server,
    /// returning their fresh user information.
    pub async fn register(&self, server: &TestServer) -> UserRegistrationResponse {
        let registration_request_model = self.into_registration_request_model();

        let registration_response = server
            .request(Method::POST, "/api/v1/users")
            .with_json_body(registration_request_model)
            .send()
            .await;

        registration_response.assert_status_equals(StatusCode::OK);

        registration_response.json_body::<UserRegistrationResponse>()
    }

    /// Returns the access token.
    pub async fn login(&self, server: &TestServer) -> String {
        let login_response = server
            .request(Method::POST, "/api/v1/login")
            .with_json_body(self.into_login_request_model())
            .send()
            .await;

        login_response.assert_status_equals(StatusCode::OK);

        login_response.json_body::<UserLoginResponse>().access_token
    }
}

/// Fetches the user information associated with the `access_token`.
pub async fn fetch_user_info(server: &TestServer, access_token: &str) -> UserInformation {
    let user_info_response = server
        .request(Method::GET, "/api/v1/users/me")
        .with_access_token(access_token)
        .send()
        .await;

    user_info_response.assert_status_equals(StatusCode::OK);
    user_info_response.assert_header_exists(header::LAST_MODIFIED);

    user_info_response.json_body::<UserInfoResponse>().user
}
