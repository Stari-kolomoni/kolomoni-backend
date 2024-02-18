use std::{collections::HashSet, time::Duration};

use chrono::Utc;
use kolomoni::api::{
    macros::construct_last_modified_header_value,
    v1::{
        login::{UserLoginRequest, UserLoginResponse},
        users::{
            all::RegisteredUsersListResponse,
            registration::{UserRegistrationRequest, UserRegistrationResponse},
            UserDisplayNameChangeRequest,
            UserDisplayNameChangeResponse,
            UserInfoResponse,
            UserPermissionsResponse,
            UserRolesResponse,
        },
    },
};
use kolomoni_test_util::prelude::*;


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SampleUser {
    Janez,
    Meta,
}

impl SampleUser {
    pub fn username(&self) -> &'static str {
        match self {
            SampleUser::Janez => "janez",
            SampleUser::Meta => "meta",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            SampleUser::Janez => "Janez Jasnovidni",
            SampleUser::Meta => "Meta Meglenska",
        }
    }

    pub fn password(&self) -> &'static str {
        match self {
            SampleUser::Janez => "janez",
            SampleUser::Meta => "meta",
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
}


pub async fn register_sample_user(server: &TestServer, sample_user: SampleUser) {
    let registration_request_model = sample_user.into_registration_request_model();

    let registration_response = server
        .request(Method::POST, "/api/v1/users")
        .with_json_body(registration_request_model)
        .send()
        .await;

    registration_response.assert_status_equals(StatusCode::OK);
}



#[tokio::test]
async fn user_registration_and_user_list_work() {
    let server = prepare_test_server_instance().await;



    // Register new user "janez".
    let time_before_registration = Utc::now();
    let registration_request = UserRegistrationRequest {
        username: "janez".to_string(),
        display_name: "Janez Veliki".to_string(),
        password: "janez".to_string(),
    };

    let registration_response = server
        .request(Method::POST, "/api/v1/users")
        .with_json_body(registration_request.clone())
        .send()
        .await;

    registration_response.assert_status_equals(StatusCode::OK);
    let new_user_info = registration_response
        .json_body::<UserRegistrationResponse>()
        .user;

    assert_eq!(
        new_user_info.username,
        registration_request.username
    );
    assert_eq!(
        new_user_info.display_name,
        registration_request.display_name
    );

    assert!(new_user_info.joined_at >= time_before_registration);
    assert!(new_user_info.last_modified_at >= time_before_registration);
    assert!(new_user_info.last_active_at >= time_before_registration);

    let time_now = Utc::now();
    assert!(new_user_info.joined_at <= time_now);
    assert!(new_user_info.last_modified_at <= time_now);
    assert!(new_user_info.last_active_at <= time_now);



    // Ensure that trying to register again fails.
    server
        .request(Method::POST, "/api/v1/users")
        .with_json_body(registration_request.clone())
        .send()
        .await
        .assert_status_equals(StatusCode::CONFLICT);



    // Ensure that trying to register under a different username,
    // but same display name, fails too.
    let conflicting_registration_request = UserRegistrationRequest {
        username: "janez2".to_string(),
        ..registration_request
    };

    server
        .request(Method::POST, "/api/v1/users")
        .with_json_body(conflicting_registration_request.clone())
        .send()
        .await
        .assert_status_equals(StatusCode::CONFLICT);



    server.give_full_permissions_to_user(new_user_info.id).await;



    // Log in our new powerful user.
    let login_request = UserLoginRequest {
        username: "janez".to_string(),
        password: "janez".to_string(),
    };

    let login_response = server
        .request(Method::POST, "/api/v1/login")
        .with_json_body(login_request)
        .send()
        .await;
    login_response.assert_status_equals(StatusCode::OK);

    let login_response_body = login_response.json_body::<UserLoginResponse>();
    let access_token = login_response_body.access_token;



    // Ensure the registration response matches the current user information endpoint.
    let current_user_info_response = server
        .request(Method::GET, "/api/v1/users/me")
        .with_access_token(&access_token)
        .send()
        .await;
    current_user_info_response.assert_status_equals(StatusCode::OK);
    current_user_info_response.assert_header_matches_value(
        header::LAST_MODIFIED,
        construct_last_modified_header_value(&new_user_info.last_modified_at).unwrap(),
    );

    let fresh_user_info = current_user_info_response
        .json_body::<UserInfoResponse>()
        .user;

    assert_eq!(new_user_info, fresh_user_info);




    // List all users, ensure it's of length 1. Register a new user and ensure the length has increased.
    let all_users_response = server
        .request(Method::GET, "/api/v1/users")
        .with_access_token(&access_token)
        .send()
        .await;
    all_users_response.assert_status_equals(StatusCode::OK);

    let all_users_response_body = all_users_response.json_body::<RegisteredUsersListResponse>();
    assert_eq!(all_users_response_body.users.len(), 1);


    server
        .request(Method::POST, "/api/v1/users")
        .with_json_body(UserRegistrationRequest {
            username: "meta".to_string(),
            display_name: "Meta".to_string(),
            password: "meta".to_string(),
        })
        .send()
        .await
        .assert_status_equals(StatusCode::OK);


    let all_users_updated_response = server
        .request(Method::GET, "/api/v1/users")
        .with_access_token(&access_token)
        .send()
        .await;
    all_users_updated_response.assert_status_equals(StatusCode::OK);

    let all_users_updated_response_body =
        all_users_updated_response.json_body::<RegisteredUsersListResponse>();
    assert_eq!(all_users_updated_response_body.users.len(), 2);
}



#[tokio::test]
async fn current_user_permissions_and_roles_work() {
    let server = prepare_test_server_instance().await;

    register_sample_user(&server, SampleUser::Janez).await;
    register_sample_user(&server, SampleUser::Meta).await;



    let login_response = server
        .request(Method::POST, "/api/v1/login")
        .with_json_body(SampleUser::Meta.into_login_request_model())
        .send()
        .await;
    login_response.assert_status_equals(StatusCode::OK);

    let login_response_body = login_response.json_body::<UserLoginResponse>();
    let access_token = login_response_body.access_token;



    let user_info_response = server
        .request(Method::GET, "/api/v1/users/me")
        .with_access_token(&access_token)
        .send()
        .await;
    user_info_response.assert_status_equals(StatusCode::OK);
    user_info_response.assert_header_exists(header::LAST_MODIFIED);

    let user_info_data = user_info_response.json_body::<UserInfoResponse>().user;



    // Test three scenarios for the `If-Modified-Since` header.

    let unmodified_response = server
        .request(Method::GET, "/api/v1/users/me")
        .with_access_token(&access_token)
        .with_header(
            header::IF_MODIFIED_SINCE,
            construct_last_modified_header_value(&user_info_data.last_modified_at).unwrap(),
        )
        .send()
        .await;
    unmodified_response.assert_status_equals(StatusCode::NOT_MODIFIED);

    let fresh_response = server
        .request(Method::GET, "/api/v1/users/me")
        .with_access_token(&access_token)
        .with_header(
            header::IF_MODIFIED_SINCE,
            construct_last_modified_header_value(
                &(user_info_data.last_modified_at - Duration::from_secs(1)),
            )
            .unwrap(),
        )
        .send()
        .await;
    fresh_response.assert_status_equals(StatusCode::OK);
    fresh_response.assert_has_json_body::<UserInfoResponse>();

    let second_unmodified_response = server
        .request(Method::GET, "/api/v1/users/me")
        .with_access_token(&access_token)
        .with_header(
            header::IF_MODIFIED_SINCE,
            construct_last_modified_header_value(
                &(user_info_data.last_modified_at + Duration::from_secs(1)),
            )
            .unwrap(),
        )
        .send()
        .await;
    second_unmodified_response.assert_status_equals(StatusCode::NOT_MODIFIED);



    // Test role system.

    server
        .request(Method::GET, "/api/v1/users/me/roles")
        .send()
        .await
        .assert_status_equals(StatusCode::UNAUTHORIZED);


    let role_list_response = server
        .request(Method::GET, "/api/v1/users/me/roles")
        .with_access_token(&access_token)
        .send()
        .await;
    role_list_response.assert_status_equals(StatusCode::OK);
    let actual_role_list = role_list_response.json_body::<UserRolesResponse>();

    assert_eq!(actual_role_list.role_names.len(), 1);
    assert_eq!(
        Role::from_name(&actual_role_list.role_names[0]).unwrap(),
        DEFAULT_USER_ROLE
    );



    // Test permission system.

    server
        .request(Method::GET, "/api/v1/users/me/permissions")
        .send()
        .await
        .assert_status_equals(StatusCode::UNAUTHORIZED);


    let user_permissions_response = server
        .request(Method::GET, "/api/v1/users/me/permissions")
        .with_access_token(&access_token)
        .send()
        .await;
    user_permissions_response.assert_status_equals(StatusCode::OK);
    let user_permissions = user_permissions_response.json_body::<UserPermissionsResponse>();

    assert_eq!(
        user_permissions
            .permissions
            .iter()
            .map(|permission_name| Permission::from_name(permission_name).unwrap())
            .collect::<HashSet<_>>(),
        HashSet::from_iter(Role::User.permissions_granted())
    );


    // Test changing the display name.
    server
        .request(Method::PATCH, "/api/v1/users/me/display_name")
        .with_json_body(UserDisplayNameChangeRequest {
            new_display_name: "Janez Mali".to_string(),
        })
        .send()
        .await
        .assert_status_equals(StatusCode::UNAUTHORIZED);

    server
        .request(Method::PATCH, "/api/v1/users/me/display_name")
        .with_json_body(UserDisplayNameChangeRequest {
            new_display_name: SampleUser::Meta.display_name().to_string(),
        })
        .with_access_token(&access_token)
        .send()
        .await
        .assert_status_equals(StatusCode::CONFLICT);


    let time_pre_display_name_update = Utc::now();
    let display_name_change_response = server
        .request(Method::PATCH, "/api/v1/users/me/display_name")
        .with_json_body(UserDisplayNameChangeRequest {
            new_display_name: "Janez Mali".to_string(),
        })
        .with_access_token(&access_token)
        .send()
        .await;
    display_name_change_response.assert_status_equals(StatusCode::OK);
    let updated_user_info = display_name_change_response
        .json_body::<UserDisplayNameChangeResponse>()
        .user;

    let time_post_display_name_update = Utc::now();

    assert_eq!(
        updated_user_info.display_name,
        "Janez Mali".to_string(),
    );
    assert!(updated_user_info.last_modified_at >= time_pre_display_name_update);
    assert!(updated_user_info.last_modified_at <= time_post_display_name_update);
}
