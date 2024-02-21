use std::{collections::HashSet, time::Duration};

use chrono::Utc;
use kolomoni::api::{
    errors::ErrorReasonResponse,
    macros::construct_last_modified_header_value,
    v1::{
        login::{UserLoginRequest, UserLoginResponse},
        users::{
            all::RegisteredUsersListResponse,
            registration::{UserRegistrationRequest, UserRegistrationResponse},
            specific::{UserRoleAddRequest, UserRoleRemoveRequest},
            UserDisplayNameChangeRequest,
            UserDisplayNameChangeResponse,
            UserInfoResponse,
            UserPermissionsResponse,
            UserRolesResponse,
        },
    },
};
use kolomoni_test_util::prelude::*;



#[tokio::test]
async fn user_registration_and_user_list_work() {
    let server = prepare_test_server_instance().await;


    // Register a new user (username "janez", display name "Janez Veliki").

    let registration_request = UserRegistrationRequest {
        username: "janez".to_string(),
        display_name: "Janez Veliki".to_string(),
        password: "janez".to_string(),
    };

    let new_user_info = {
        let time_before_registration = Utc::now();
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

        new_user_info
    };




    {
        // Trying to register with an existing username or display name
        // should fail with `409 Conflict`.
        server
            .request(Method::POST, "/api/v1/users")
            .with_json_body(registration_request.clone())
            .send()
            .await
            .assert_status_equals(StatusCode::CONFLICT);
    }



    // Ensure that trying to register with a conflicting display name fails.
    {
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
    }



    server.give_full_permissions_to_user(new_user_info.id).await;



    // Log in our new powerful user.

    let access_token = {
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

        login_response_body.access_token
    };



    // Ensure the registration response matches the current user information endpoint.

    {
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
    }




    // List all users; ensure there is only one.
    // Then register a new user and ensure the length has increased to 2.

    server
        .request(Method::GET, "/api/v1/users")
        .send()
        .await
        .assert_status_equals(StatusCode::UNAUTHORIZED);


    {
        let all_users_response = server
            .request(Method::GET, "/api/v1/users")
            .with_access_token(&access_token)
            .send()
            .await;
        all_users_response.assert_status_equals(StatusCode::OK);

        let all_users_response_body = all_users_response.json_body::<RegisteredUsersListResponse>();
        assert_eq!(all_users_response_body.users.len(), 1);
    }


    {
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
    }


    {
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
}



#[tokio::test]
async fn current_user_permissions_and_roles_work() {
    let server = prepare_test_server_instance().await;

    register_sample_user(&server, SampleUser::Janez).await;
    register_sample_user(&server, SampleUser::Meta).await;

    let access_token = login_sample_user(&server, SampleUser::Meta).await;



    let sample_user_info = {
        let user_info_response = server
            .request(Method::GET, "/api/v1/users/me")
            .with_access_token(&access_token)
            .send()
            .await;
        user_info_response.assert_status_equals(StatusCode::OK);
        user_info_response.assert_header_exists(header::LAST_MODIFIED);

        user_info_response.json_body::<UserInfoResponse>().user
    };



    // Test three scenarios for the `If-Modified-Since` header.

    {
        // If the user hasn't changed since `If-Modified-Since`, the server should return
        // a `304 Not Modified`.
        let unmodified_response = server
            .request(Method::GET, "/api/v1/users/me")
            .with_access_token(&access_token)
            .with_header(
                header::IF_MODIFIED_SINCE,
                construct_last_modified_header_value(&sample_user_info.last_modified_at).unwrap(),
            )
            .send()
            .await;

        unmodified_response.assert_status_equals(StatusCode::NOT_MODIFIED);
    }

    {
        // If the user hasn't changed since `If-Modified-Since`, the server should return
        // a `304 Not Modified`.
        let second_unmodified_response = server
            .request(Method::GET, "/api/v1/users/me")
            .with_access_token(&access_token)
            .with_header(
                header::IF_MODIFIED_SINCE,
                construct_last_modified_header_value(
                    &(sample_user_info.last_modified_at + Duration::from_secs(1)),
                )
                .unwrap(),
            )
            .send()
            .await;

        second_unmodified_response.assert_status_equals(StatusCode::NOT_MODIFIED);
    }

    {
        // If the user has changed since the specified time, the server should return
        // fresh user information.
        let fresh_response = server
            .request(Method::GET, "/api/v1/users/me")
            .with_access_token(&access_token)
            .with_header(
                header::IF_MODIFIED_SINCE,
                construct_last_modified_header_value(
                    &(sample_user_info.last_modified_at - Duration::from_secs(1)),
                )
                .unwrap(),
            )
            .send()
            .await;

        fresh_response.assert_status_equals(StatusCode::OK);
        fresh_response.assert_has_json_body::<UserInfoResponse>();
    }



    // Test role system.

    {
        // Seeing your roles requires that you authenticate in the first place.
        server
            .request(Method::GET, "/api/v1/users/me/roles")
            .send()
            .await
            .assert_status_equals(StatusCode::UNAUTHORIZED);
    }

    {
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
    }



    // Test permission system.

    {
        // Seeing your permissions requires that you authenticate in the first place.
        server
            .request(Method::GET, "/api/v1/users/me/permissions")
            .send()
            .await
            .assert_status_equals(StatusCode::UNAUTHORIZED);
    }

    {
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
    }



    // Test changing the display name.
    {
        // Changing your display name requires that you authenticate in the first place.
        server
            .request(Method::PATCH, "/api/v1/users/me/display_name")
            .with_json_body(UserDisplayNameChangeRequest {
                new_display_name: "Janez Mali".to_string(),
            })
            .send()
            .await
            .assert_status_equals(StatusCode::UNAUTHORIZED);

        // Your new display name must not conflict with another user.
        server
            .request(Method::PATCH, "/api/v1/users/me/display_name")
            .with_json_body(UserDisplayNameChangeRequest {
                new_display_name: SampleUser::Meta.display_name().to_string(),
            })
            .with_access_token(&access_token)
            .send()
            .await
            .assert_status_equals(StatusCode::CONFLICT);
    }

    {
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
}


#[tokio::test]
async fn specific_user_operations_work() {
    let server = prepare_test_server_instance().await;

    register_sample_user(&server, SampleUser::Janez).await;
    register_sample_user(&server, SampleUser::Meta).await;

    let sample_user_access_token = login_sample_user(&server, SampleUser::Meta).await;

    let sample_user_info = get_sample_user_info(&server, &sample_user_access_token).await;
    server
        .give_full_permissions_to_user(sample_user_info.id)
        .await;


    // Find user with username "janez".
    let janez_user_info = {
        let all_users_response = server
            .request(Method::GET, "/api/v1/users")
            .with_access_token(&sample_user_access_token)
            .send()
            .await;
        all_users_response.assert_status_equals(StatusCode::OK);
        let user_list = all_users_response.json_body::<RegisteredUsersListResponse>();

        user_list
            .users
            .iter()
            .find(|user| user.username == "janez")
            .unwrap()
            .clone()
    };



    /***
     * Test getting specific user information.
     */

    {
        // Non-existent user IDs should return `404 Not Found`.
        server
            .request(Method::GET, "/api/v1/users/238429")
            .send()
            .await
            .assert_status_equals(StatusCode::NOT_FOUND);
    }

    {
        // Individual users should be accessible without authentication.
        server
            .request(
                Method::GET,
                format!("/api/v1/users/{}", janez_user_info.id),
            )
            .send()
            .await
            .assert_status_equals(StatusCode::OK);
    }

    {
        // Individual users should also be accessible *with* authentication.
        let janez_info_response = server
            .request(
                Method::GET,
                format!("/api/v1/users/{}", janez_user_info.id),
            )
            .with_access_token(&sample_user_access_token)
            .send()
            .await;

        janez_info_response.assert_status_equals(StatusCode::OK);

        let second_janez_user_info = janez_info_response.json_body::<UserInfoResponse>().user;

        assert_eq!(janez_user_info, second_janez_user_info);
    }



    /***
     * Test getting specific user roles and permissions.
     */

    {
        // Requesting roles for a non-existent user should fail with `404 Not Found`.
        server
            .request(Method::GET, "/api/v1/users/238429/roles")
            .send()
            .await
            .assert_status_equals(StatusCode::NOT_FOUND);
    }

    {
        // Individual users' roles should be accessible without authentication.
        let janez_roles_response = server
            .request(
                Method::GET,
                format!("/api/v1/users/{}/roles", janez_user_info.id),
            )
            .send()
            .await;

        janez_roles_response.assert_status_equals(StatusCode::OK);
        janez_roles_response.assert_has_json_body::<UserRolesResponse>();
    }

    let janez_roles = {
        // Individual users' roles should also be accessible *with* authentication.
        let janez_roles_response = server
            .request(
                Method::GET,
                format!("/api/v1/users/{}/roles", janez_user_info.id),
            )
            .with_access_token(&sample_user_access_token)
            .send()
            .await;

        janez_roles_response.assert_status_equals(StatusCode::OK);

        let janez_role_names = janez_roles_response
            .json_body::<UserRolesResponse>()
            .role_names;

        assert_eq!(
            janez_role_names,
            vec![DEFAULT_USER_ROLE.name().to_string()]
        );

        janez_role_names
            .into_iter()
            .map(|role_name| Role::from_name(&role_name).unwrap())
            .collect::<HashSet<_>>()
    };


    {
        // Requesting permissions should require authentication,
        // regardless of whether the user exists or not.
        server
            .request(Method::GET, "/api/v1/users/238429/permissions")
            .send()
            .await
            .assert_status_equals(StatusCode::UNAUTHORIZED);

        // Properly requesting permissions for a non-existent user
        // should fail with `404 Not Found`.
        server
            .request(Method::GET, "/api/v1/users/238429/permissions")
            .with_access_token(&sample_user_access_token)
            .send()
            .await
            .assert_status_equals(StatusCode::NOT_FOUND);
    }

    {
        let janez_permissions_response = server
            .request(
                Method::GET,
                format!("/api/v1/users/{}/permissions", janez_user_info.id),
            )
            .with_access_token(&sample_user_access_token)
            .send()
            .await;

        janez_permissions_response.assert_status_equals(StatusCode::OK);
        let janez_permission_names = janez_permissions_response
            .json_body::<UserPermissionsResponse>()
            .permissions;


        let janez_expected_permission_set = janez_roles
            .into_iter()
            .flat_map(|role| role.permissions_granted())
            .collect::<HashSet<_>>();

        let janez_actual_permission_set = janez_permission_names
            .iter()
            .map(|permission_name| Permission::from_name(permission_name).unwrap())
            .collect::<HashSet<_>>();

        assert_eq!(
            janez_expected_permission_set,
            janez_actual_permission_set
        );
    }


    /***
     * Test modifying another user's display name.
     */

    {
        // Failing to send a JSON body when trying to modify the display
        // name should fail with `400 Bad Request`, even if authentication is not provided.
        server
            .request(Method::PATCH, "/api/v1/users/238429/display_name")
            .send()
            .await
            .assert_status_equals(StatusCode::BAD_REQUEST);

        // Changing a user's display name should require authentication.
        server
            .request(Method::PATCH, "/api/v1/users/238429/display_name")
            .with_json_body(UserDisplayNameChangeRequest {
                new_display_name: "Some Display Name".to_string(),
            })
            .send()
            .await
            .assert_status_equals(StatusCode::UNAUTHORIZED);

        // Attempting to change a non-existent user's display name should fail with `404 Not Found`.
        let not_found_response = server
            .request(Method::PATCH, "/api/v1/users/238429/display_name")
            .with_json_body(UserDisplayNameChangeRequest {
                new_display_name: "Some Display Name".to_string(),
            })
            .with_access_token(&sample_user_access_token)
            .send()
            .await;

        not_found_response.assert_status_equals(StatusCode::NOT_FOUND);
        not_found_response.assert_has_json_body::<ErrorReasonResponse>();


        // In case of display name conflict, `409 Conflict` should be returned.
        server
            .request(
                Method::PATCH,
                format!(
                    "/api/v1/users/{}/display_name",
                    janez_user_info.id
                ),
            )
            .with_json_body(UserDisplayNameChangeRequest {
                new_display_name: SampleUser::Janez.display_name().to_string(),
            })
            .with_access_token(&sample_user_access_token)
            .send()
            .await
            .assert_status_equals(StatusCode::CONFLICT);
    }

    {
        let time_pre_display_name_change = Utc::now();
        let display_name_change_response = server
            .request(
                Method::PATCH,
                format!(
                    "/api/v1/users/{}/display_name",
                    janez_user_info.id
                ),
            )
            .with_json_body(UserDisplayNameChangeRequest {
                new_display_name: "Janez Koglot".to_string(),
            })
            .with_access_token(&sample_user_access_token)
            .send()
            .await;

        display_name_change_response.assert_status_equals(StatusCode::OK);

        let third_janez_user_info = display_name_change_response
            .json_body::<UserDisplayNameChangeResponse>()
            .user;

        assert_eq!(third_janez_user_info.display_name, "Janez Koglot");

        let time_post_display_name_change = Utc::now();
        assert!(third_janez_user_info.last_modified_at >= time_pre_display_name_change);
        assert!(third_janez_user_info.last_modified_at <= time_post_display_name_change);
    }


    /***
     * Test adding another user's roles.
     */

    {
        // Failing to provide a JSON body with the list of roles to add
        // should fail with `400 Bad Request`.
        server
            .request(Method::POST, "/api/v1/users/238429/roles")
            .send()
            .await
            .assert_status_equals(StatusCode::BAD_REQUEST);

        // Adding roles to a user should require authentication.
        server
            .request(Method::POST, "/api/v1/users/238429/roles")
            .with_json_body(UserRoleAddRequest {
                roles_to_add: vec![Role::Administrator.name().to_string()],
            })
            .send()
            .await
            .assert_status_equals(StatusCode::UNAUTHORIZED);

        // Trying to add roles to a non-existent user should fail with `404 Not Found`.
        server
            .request(Method::POST, "/api/v1/users/238429/roles")
            .with_json_body(UserRoleAddRequest {
                roles_to_add: vec![Role::Administrator.name().to_string()],
            })
            .with_access_token(&sample_user_access_token)
            .send()
            .await
            .assert_status_equals(StatusCode::NOT_FOUND);

        // If any of the provided role names are invalid, the request should fail
        // with `400 Bad Request`.
        server
            .request(
                Method::POST,
                format!("/api/v1/users/{}/roles", janez_user_info.id),
            )
            .with_json_body(UserRoleAddRequest {
                roles_to_add: vec!["non-existent-role-name".to_string()],
            })
            .with_access_token(&sample_user_access_token)
            .send()
            .await
            .assert_status_equals(StatusCode::BAD_REQUEST);
    }

    {
        // We should not be able to modify ourselves.
        server
            .request(
                Method::POST,
                format!("/api/v1/users/{}/roles", sample_user_info.id),
            )
            .with_json_body(UserRoleAddRequest {
                roles_to_add: vec![Role::Administrator.name().to_string()],
            })
            .with_access_token(&sample_user_access_token)
            .send()
            .await
            .assert_status_equals(StatusCode::FORBIDDEN);
    }

    {
        let previous_roles = {
            let role_list_response = server
                .request(
                    Method::GET,
                    format!("/api/v1/users/{}/roles", janez_user_info.id),
                )
                .with_access_token(&sample_user_access_token)
                .send()
                .await;

            role_list_response.assert_status_equals(StatusCode::OK);

            role_list_response
                .json_body::<UserRolesResponse>()
                .role_names
                .into_iter()
                .map(|role_name| Role::from_name(&role_name).unwrap())
                .collect::<HashSet<Role>>()
        };

        assert!(!previous_roles.contains(&Role::Administrator));


        // Our sample user (Meta) has all permissions at the moment, so
        // giving the admin role should not fail.
        let add_role_response = server
            .request(
                Method::POST,
                format!("/api/v1/users/{}/roles", janez_user_info.id),
            )
            .with_json_body(UserRoleAddRequest {
                roles_to_add: vec![Role::Administrator.name().to_string()],
            })
            .with_access_token(&sample_user_access_token)
            .send()
            .await;

        add_role_response.assert_status_equals(StatusCode::OK);


        let updated_roles = add_role_response
            .json_body::<UserRolesResponse>()
            .role_names
            .into_iter()
            .map(|role_name| Role::from_name(&role_name).unwrap())
            .collect::<HashSet<Role>>();

        assert!(updated_roles.contains(&Role::Administrator));
    }


    /***
     * Test removing roles from users
     */

    {
        // Failing to provide a list of roles to remove should fail with `400 Bad Request`.
        server
            .request(Method::DELETE, "/api/v1/users/238429/roles")
            .send()
            .await
            .assert_status_equals(StatusCode::BAD_REQUEST);

        // Authentication should be required to remove roles from users.
        server
            .request(Method::DELETE, "/api/v1/users/238429/roles")
            .with_json_body(UserRoleRemoveRequest {
                roles_to_remove: vec![Role::Administrator.name().to_string()],
            })
            .send()
            .await
            .assert_status_equals(StatusCode::UNAUTHORIZED);

        // Trying to remove roles from a non-existent user should fail with `404 Not Found`.
        server
            .request(Method::DELETE, "/api/v1/users/238429/roles")
            .with_json_body(UserRoleRemoveRequest {
                roles_to_remove: vec![Role::Administrator.name().to_string()],
            })
            .with_access_token(&sample_user_access_token)
            .send()
            .await
            .assert_status_equals(StatusCode::NOT_FOUND);

        // Providing an invalid role name should fail with `400 Bad Request`.
        server
            .request(
                Method::DELETE,
                format!("/api/v1/users/{}/roles", janez_user_info.id),
            )
            .with_json_body(UserRoleRemoveRequest {
                roles_to_remove: vec!["non-existent-role-name".to_string()],
            })
            .with_access_token(&sample_user_access_token)
            .send()
            .await
            .assert_status_equals(StatusCode::BAD_REQUEST);

        // Removing roles from yourself should not be allowed.
        server
            .request(
                Method::DELETE,
                format!("/api/v1/users/{}/roles", sample_user_info.id),
            )
            .with_json_body(UserRoleRemoveRequest {
                roles_to_remove: vec![Role::Administrator.name().to_string()],
            })
            .with_access_token(&sample_user_access_token)
            .send()
            .await
            .assert_status_equals(StatusCode::FORBIDDEN);
    }

    {
        let previous_roles = {
            let role_list_response = server
                .request(
                    Method::GET,
                    format!("/api/v1/users/{}/roles", janez_user_info.id),
                )
                .with_access_token(&sample_user_access_token)
                .send()
                .await;

            role_list_response.assert_status_equals(StatusCode::OK);

            role_list_response
                .json_body::<UserRolesResponse>()
                .role_names
                .into_iter()
                .map(|role_name| Role::from_name(&role_name).unwrap())
                .collect::<HashSet<Role>>()
        };

        assert!(previous_roles.contains(&Role::Administrator));


        let role_removal_response = server
            .request(
                Method::DELETE,
                format!("/api/v1/users/{}/roles", janez_user_info.id),
            )
            .with_access_token(&sample_user_access_token)
            .with_json_body(UserRoleRemoveRequest {
                roles_to_remove: vec![Role::Administrator.name().to_string()],
            })
            .send()
            .await;

        role_removal_response.assert_status_equals(StatusCode::OK);


        let updated_roles = role_removal_response
            .json_body::<UserRolesResponse>()
            .role_names
            .into_iter()
            .map(|role_name| Role::from_name(&role_name).unwrap())
            .collect::<HashSet<Role>>();

        assert!(!updated_roles.contains(&Role::Administrator));
    }


    /***
     * Test that users can't give roles they don't have.
     */

    // This resets our logged-in user from an admin to a normal user.
    server
        .reset_user_permissions_to_normal(sample_user_info.id)
        .await;

    {
        let sample_user_roles = {
            let role_list_response = server
                .request(Method::GET, "/api/v1/users/me/roles")
                .with_access_token(&sample_user_access_token)
                .send()
                .await;

            role_list_response.assert_status_equals(StatusCode::OK);

            role_list_response
                .json_body::<UserRolesResponse>()
                .role_names
                .into_iter()
                .map(|role_name| Role::from_name(&role_name).unwrap())
                .collect::<HashSet<Role>>()
        };

        assert_eq!(
            sample_user_roles,
            HashSet::from_iter([DEFAULT_USER_ROLE])
        );
    }

    {
        // Trying to add e.g. an administrator role to another user without having
        // it yourself should fail with `403 Forbidden`.
        let forbidden_role_add_response = server
            .request(
                Method::POST,
                format!("/api/v1/users/{}/roles", janez_user_info.id,),
            )
            .with_access_token(&sample_user_access_token)
            .with_json_body(UserRoleAddRequest {
                roles_to_add: vec![Role::Administrator.name().to_string()],
            })
            .send()
            .await;

        forbidden_role_add_response.assert_status_equals(StatusCode::FORBIDDEN);
    }


    /***
     * Test that users can't remove roles they don't have.
     */

    // This will give the Janez sample user all roles, including the administrator role.
    server
        .give_full_permissions_to_user(janez_user_info.id)
        .await;

    {
        let janez_user_roles = {
            let role_list_response = server
                .request(
                    Method::GET,
                    format!("/api/v1/users/{}/roles", janez_user_info.id),
                )
                .send()
                .await;

            role_list_response.assert_status_equals(StatusCode::OK);

            role_list_response
                .json_body::<UserRolesResponse>()
                .role_names
                .into_iter()
                .map(|role_name| Role::from_name(&role_name).unwrap())
                .collect::<HashSet<Role>>()
        };

        assert_eq!(
            janez_user_roles,
            HashSet::from_iter([Role::Administrator, Role::User])
        );
    }

    {
        // Trying to remove another user's e.g. administrator role should be impossible
        // if you do not have that role yourself.
        let forbidden_role_remove_response = server
            .request(
                Method::DELETE,
                format!("/api/v1/users/{}/roles", janez_user_info.id),
            )
            .with_access_token(&sample_user_access_token)
            .with_json_body(UserRoleRemoveRequest {
                roles_to_remove: vec![Role::Administrator.name().to_string()],
            })
            .send()
            .await;

        forbidden_role_remove_response.assert_status_equals(StatusCode::FORBIDDEN);
    }
}
