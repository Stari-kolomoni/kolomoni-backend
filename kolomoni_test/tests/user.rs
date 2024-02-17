use chrono::Utc;
use kolomoni::api::{
    macros::construct_last_modified_header_value,
    v1::{
        login::{UserLoginRequest, UserLoginResponse},
        users::{
            all::RegisteredUsersListResponse,
            registration::{UserRegistrationRequest, UserRegistrationResponse},
            UserInfoResponse,
        },
    },
};
use kolomoni_test_util::prelude::*;


#[tokio::test]
async fn basic_user_operations_work() {
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
        .with_authentication_token(&access_token)
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
        .with_authentication_token(&access_token)
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
        .with_authentication_token(&access_token)
        .send()
        .await;
    all_users_updated_response.assert_status_equals(StatusCode::OK);

    let all_users_updated_response_body =
        all_users_updated_response.json_body::<RegisteredUsersListResponse>();
    assert_eq!(all_users_updated_response_body.users.len(), 2);

    // Test user modifications (display name, roles).
}
