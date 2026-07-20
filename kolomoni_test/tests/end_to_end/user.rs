use chrono::Utc;
use kolomoni_api_client::{
    api::{
        auth::{AuthenticationApiAnonymousEndpoints, UserRegistration, UserRegistrationError},
        users::{
            current::{CurrentUserEndpoints, CurrentUserReadError},
            ConditionalUserInfo,
            SpecificUserAuthenticatedEndpoints,
            SpecificUserUnauthenticatedEndpoints,
            UserDisplayNameUpdateError,
        },
    },
    client::RequestError,
};
use kolomoni_core::{
    ids::UserId,
    roles::{Role, DEFAULT_USER_ROLE_SET},
};
use kolomoni_test_core::macros::test as kolomoni_test;
use kolomoni_test_core::{macros::assert_matches, prelude::*};
use uuid::Uuid;


/// Tests the following requirements:
/// - Registering a new user with an existing username MUST fail.
/// - Registering a new user with an existing display name MUST fail.
/// - A user with a unique username and display name SHOULD be able to register successfully.
/// - If the If-Modified-Since header is present, the server MUST respect it when
///   fetching the current user information.
/// - An unauthenticated user MUST NOT be able to retrieve their own information (since they are unauthenticated).
/// - An authenticated user MUST be able to sucesffully retrieve their own information.
/// - An authenticated user MUST appear in the full user list.
///
/// (These requirements are not exhaustive.)
#[kolomoni_test]
pub async fn user_registration_and_listing_works() {
    let client = fresh_test_client!();

    // Register a new user, then manually fetch the user and ensure they are the same.
    let time_before_registration = Utc::now();
    let new_user = SampleUser::Janez.register(&client).await;
    let time_after_registration = Utc::now();

    let new_fetched_user = client
        .users()
        .get_user_information_by_user_id(new_user.id)
        .send()
        .await
        .unwrap_or_else(|error| {
            panic!(
                "failed to obtain user information of the just-registered sample user: {}",
                error
            )
        });

    assert_eq!(new_user, new_fetched_user);

    assert_eq!(
        new_user.display_name,
        SampleUser::Janez.display_name()
    );
    assert_eq!(new_user.username, SampleUser::Janez.username());

    assert_eq!(new_user.joined_at, new_user.last_active_at);
    assert_eq!(new_user.joined_at, new_user.last_modified_at);

    assert!(new_user.joined_at >= time_before_registration);
    assert!(new_user.joined_at <= time_after_registration);


    // Give user administrator and prepare for further tests.
    client
        .testing()
        .give_user_administrator_role(new_user.id)
        .await;

    let authentication = SampleUser::Janez.login(&client).await;
    let authenticated_client = client.with_authentication(authentication);


    // An unauthenticated user MUST NOT be able to retrieve their own information
    // due to missing permissions and since they are unauthenticated anyway.
    {
        let unrestricted_client = client.to_unrestricted_test_client();

        let failed_current_user_info = unrestricted_client
            .current_user()
            .get_current_user_information()
            .send()
            .await
            .unwrap_err();

        assert_matches!(
            failed_current_user_info,
            CurrentUserReadError::RequestError(RequestError::MissingAuthentication)
        );
    }

    // Ensure the information about the currently authenticated user is correct.
    {
        let current_user_info = authenticated_client
            .current_user()
            .get_current_user_information()
            .send()
            .await
            .unwrap_or_else(|error| {
                panic!("failed to get currently authenticated user's information: {error}")
            });

        assert_eq!(current_user_info.id, new_user.id);

        assert_eq!(current_user_info.username, new_user.username);
        assert_eq!(
            current_user_info.display_name,
            new_user.display_name
        );

        assert_eq!(current_user_info.joined_at, new_user.joined_at);
    }

    // If the If-Modified-Since header is present, the server MUST respect it when
    // fetching the current user information.
    {
        let current_user_info = authenticated_client
            .current_user()
            .get_current_user_information_if_modified_since(&time_before_registration)
            .send()
            .await
            .unwrap_or_else(|error| {
                panic!("failed to conditionally get currently authenticated user's information: {error}")
            });

        assert_matches!(
            current_user_info,
            ConditionalUserInfo::Modified { .. }
        );

        let unmodified_current_user_info = authenticated_client
            .current_user()
            .get_current_user_information_if_modified_since(&time_after_registration)
            .send()
            .await
            .unwrap_or_else(|error| {
                panic!("failed to conditionally get currently authenticated user's information: {error}")
            });

        let ConditionalUserInfo::Unmodified {
            last_modified_at: server_provided_last_modified_at,
        } = unmodified_current_user_info
        else {
            panic!(
                "failed to conditionally get currently authenticated user's information: \
                server should have indicated that the user hasn't been modified"
            )
        };

        assert_eq!(
            server_provided_last_modified_at,
            new_user.last_modified_at
        );
    }


    // Ensure the registered user appears in the full user list.
    {
        let all_registered_users = authenticated_client
            .users()
            .get_all_registered_users()
            .send()
            .await
            .unwrap_or_else(|error| panic!("failed to fetch all registered users: {}", error));

        assert!(all_registered_users.len() == 1);

        assert_eq!(all_registered_users[0].id, new_user.id);
        assert_eq!(
            all_registered_users[0].display_name,
            new_user.display_name
        );
        assert_eq!(
            all_registered_users[0].username,
            new_user.username
        );
        assert_eq!(
            all_registered_users[0].joined_at,
            new_user.joined_at
        );
        assert_eq!(
            all_registered_users[0].last_modified_at,
            new_user.last_modified_at
        );
    }


    // Ensure a new registration is added to the full user list
    {
        SampleUser::Kira.register(&client).await;

        let all_registered_users = authenticated_client
            .users()
            .get_all_registered_users()
            .send()
            .await
            .unwrap_or_else(|error| {
                panic!(
                    "failed to fetch all registered user again: {}",
                    error
                )
            });

        assert!(all_registered_users.len() == 2);
    }


    // Registering with an existing username should fail.
    {
        let re_registration_error = client
            .authentication()
            .register_user(UserRegistration {
                username: SampleUser::Janez.username().to_owned(),
                display_name: "SAMPLE".to_owned(),
                password: SampleUser::Janez.password().to_owned(),
            })
            .send()
            .await
            .unwrap_err();

        assert_matches!(
            re_registration_error,
            UserRegistrationError::UsernameAlreadyExists
        );
    }


    // Registering with an existing display name should fail.
    {
        let re_registration_error = client
            .authentication()
            .register_user(UserRegistration {
                username: "SAMPLE".to_owned(),
                display_name: SampleUser::Janez.display_name().to_owned(),
                password: SampleUser::Janez.password().to_owned(),
            })
            .send()
            .await
            .unwrap_err();

        assert_matches!(
            re_registration_error,
            UserRegistrationError::DisplayNameAlreadyExists
        );
    }
}


/// Tests the following requirements:
/// - Display name changes MUST require authentication.
/// - Display name changes MUST fail when there is no such target user.
/// - Display name changes MUST fail when there is a display name collision.
/// - Display name changes MUST succeed in other cases and also update the user's last_modified_at.
///
/// (These requirements are not exhaustive.)
#[kolomoni_test]
pub async fn listing_user_permissions_and_roles_works() {
    let client = fresh_test_client!();
    let unrestricted_client = client.to_unrestricted_test_client();

    let janez_user_info = SampleUser::Janez.register(&client).await;
    let meta_user_info = SampleUser::Meta.register(&client).await;

    let authentication = SampleUser::Janez.login(&client).await;
    let authenticated_client = client.with_authentication(authentication);

    let dummy_user_id = UserId::new(Uuid::from_u128(0));

    // Display name changes MUST require authentication.
    {
        let failed_display_name_change_err = unrestricted_client
            .users()
            .update_user_display_name_by_user_id(janez_user_info.id, "placeholder")
            .send()
            .await
            .unwrap_err();

        assert_matches!(
            failed_display_name_change_err,
            UserDisplayNameUpdateError::RequestError(RequestError::MissingAuthentication)
        );
    }


    // Display name changes MUST fail when there is no such target user.
    {
        let failed_display_name_change_err = authenticated_client
            .users()
            .update_user_display_name_by_user_id(dummy_user_id, "placeholder")
            .send()
            .await
            .unwrap_err();

        assert_matches!(
            failed_display_name_change_err,
            UserDisplayNameUpdateError::NotFound
        );
    }


    // Display name changes MUST fail when there is a display name collision.
    {
        let failed_display_name_change_err = authenticated_client
            .users()
            .update_user_display_name_by_user_id(meta_user_info.id, janez_user_info.display_name)
            .send()
            .await
            .unwrap_err();

        assert_matches!(
            failed_display_name_change_err,
            UserDisplayNameUpdateError::DisplayNameAlreadyExists
        );
    }


    // Display name changes MUST succeed in other cases and also update
    // the user's last_modified_at.
    {
        let time_before_display_name_change = Utc::now();

        let updated_user = authenticated_client
            .users()
            .update_user_display_name_by_user_id(
                meta_user_info.id,
                format!("{}2", meta_user_info.display_name),
            )
            .send()
            .await
            .unwrap();

        let time_after_display_name_change = Utc::now();

        assert!(updated_user.last_modified_at > meta_user_info.last_modified_at);
        assert!(updated_user.last_modified_at > time_before_display_name_change);
        assert!(updated_user.last_modified_at < time_after_display_name_change);
    }
}


/// Tests the following requirements:
/// - Fetching your own roles MUST fail without authentication.
/// - Fetching your own permissions MUST fail without authentication.
///
/// (These requirements are not exhaustive.)
#[kolomoni_test]
pub async fn user_roles_and_permissions_require_authentication() {
    let client = fresh_test_client!();
    let unrestricted_client = client.to_unrestricted_test_client();

    // Fetching your own roles MUST fail without authentication.
    {
        let current_user_roles_err = unrestricted_client
            .current_user()
            .get_current_user_roles()
            .send()
            .await
            .unwrap_err();

        assert_matches!(
            current_user_roles_err,
            CurrentUserReadError::RequestError(RequestError::MissingAuthentication)
        );
    }

    // Fetching your own permissions MUST fail without authentication.
    {
        let current_user_permissions_err = unrestricted_client
            .current_user()
            .get_current_user_effective_permissions()
            .send()
            .await
            .unwrap_err();

        assert_matches!(
            current_user_permissions_err,
            CurrentUserReadError::RequestError(RequestError::MissingAuthentication)
        );
    }
}


/// Tests the following requirements for a normal user:
/// - Fetching your own roles MUST return the correct set of roles.
/// - Fetching your own permissions MUST return the correct set of roles.
///
/// (These requirements are not exhaustive.)
#[kolomoni_test]
async fn default_user_roles_and_permissions_work() {
    let client = fresh_test_client!();

    let authentication = SampleUser::Janez.login(&client).await;
    let authenticated_client = client.with_authentication(authentication);

    let default_set_of_roles = (*DEFAULT_USER_ROLE_SET).clone();

    // Fetching your own roles MUST return the correct set of roles.
    {
        let user_roles = authenticated_client
            .current_user()
            .get_current_user_roles()
            .send()
            .await
            .unwrap();

        assert_eq!(user_roles, default_set_of_roles);
    }

    // Fetching your own permissions MUST return the correct set of roles.
    {
        let user_permissions = authenticated_client
            .current_user()
            .get_current_user_effective_permissions()
            .send()
            .await
            .unwrap();

        let default_set_of_permissions = default_set_of_roles.to_granted_permission_set();

        assert_eq!(user_permissions, default_set_of_permissions)
    }
}


/// Tests the following requirements for an administrator:
/// - Fetching your own roles MUST return the correct set of roles.
/// - Fetching your own permissions MUST return the correct set of roles.
///
/// (These requirements are not exhaustive.)
#[kolomoni_test]
async fn administrator_user_roles_and_permissions_work() {
    let client = fresh_test_client!();

    let janez_user_info = SampleUser::Janez.register(&client).await;

    let authentication = SampleUser::Janez.login(&client).await;
    let authenticated_client = client.with_authentication(authentication);

    let default_set_of_admin_roles = {
        let mut role_set = (*DEFAULT_USER_ROLE_SET).clone();
        role_set.add_role(Role::Administrator);

        role_set
    };

    client
        .testing()
        .give_user_administrator_role(janez_user_info.id)
        .await;

    // Fetching your own roles MUST return the correct set of roles.
    {
        let admin_user_roles = authenticated_client
            .current_user()
            .get_current_user_roles()
            .send()
            .await
            .unwrap();

        assert_eq!(admin_user_roles, default_set_of_admin_roles);
    }

    // Fetching your own permissions MUST return the correct set of roles.
    {
        let admin_user_permissions = authenticated_client
            .current_user()
            .get_current_user_effective_permissions()
            .send()
            .await
            .unwrap();

        let effective_set_of_admin_permissions =
            default_set_of_admin_roles.to_granted_permission_set();

        assert_eq!(
            admin_user_permissions,
            effective_set_of_admin_permissions
        );
    }
}


/// Tests the following requirements:
/// - Queries for a non-existent user MUST fail.
/// - Querying information about an individual user by ID SHOULD not require authentication.
/// - Querying information about an individual user by ID MUST also be accessible with authentication.
/// - Requesting a role set for a non-existent user MUST fail.
/// - Requesting a role set for a specific user by ID MUST fail without authentication.
/// - Requesting a role set for a specific user by ID SHOULD work with authentication.
/// - Requesting a permission set for a non-existent user MUST fail.
/// - Requesting a permission set for a specific user by ID MUST fail without authentication.
/// - Requesting a permission set for a specific user by ID SHOULD work with authentication.
/// - Attempting to update a non-existent user's display name MUST fail.
/// - Updating another user's display name MUST fail without authentication.
/// - Updating another user's display name (with authentication) to an existing name MUST fail.
/// - Updating another user's display name (with authentication) MUST fail if the caller does not have enough permissions.
/// - Updating another user's display name (with authentication) MUST work if the caller has enough permissions.
/// - Updating your own user's display name (with authentication) through a non-"/me/*" endpoint MUST fail.
/// - Adding a role to non-existent user's role set MUST fail.
/// - Adding a role to another user's role set MUST fail without authentication.
/// - Adding a role to another user's role set (with authentication) MUST fail if the caller does not have all the implied permissions they want to add to that user.
/// - Adding a role to another user's role set (with authentication) MUST fail if any role name is invalid.
/// - Adding a role to another user's role set (with authentication) MUST succeed if the caller has all the implied permissions they want to add to that user.
/// - Adding a role to your own user's role set (with authentication) through the generic user role update endpoint MUST fail.
/// - Removing a role from non-existent user's role set MUST fail.
/// - Removing a role from another user's role set MUST fail without authentication.
/// - Removing a role from another user's role set (with authentication) MUST fail if the caller does not have all the implied permissions they want to remove from that user.
/// - Removing a role from another user's role set (with authentication) MUST fail if any role name is invalid.
/// - Removing a role from another user's role set (with authentication) MUST succeed if the caller has all the implied permissions they want to remove from that user.
/// - Removing a role from your own user's role set (with authentication) through the generic user role update endpoint MUST fail.
#[kolomoni_test]
async fn asd() {
    let client = fresh_test_client!();
    let unrestricted_client = client.to_unrestricted_test_client();

    let janez_user_info = SampleUser::Janez.register(&client).await;
    let meta_user_info = SampleUser::Meta.register(&client).await;

    let authentication = SampleUser::Janez.login(&client).await;
    let authenticated_client = client.with_authentication(authentication);

    let dummy_user_id = UserId::new(Uuid::from_u128(0));

    // Queries for a non-existent user MUST fail.
    // TODO

    // Querying information about an individual user by ID SHOULD not require authentication.
    // TODO

    // Querying information about an individual user by ID MUST also be accessible with authentication.
    // TODO

    // Requesting a role set for a non-existent user MUST fail.
    // TODO

    // Requesting a role set for a specific user by ID MUST fail without authentication.
    // TODO

    // Requesting a role set for a specific user by ID SHOULD work with authentication.
    // TODO

    // Requesting a permission set for a non-existent user MUST fail.
    // TODO

    // Requesting a permission set for a specific user by ID MUST fail without authentication.
    // TODO

    // Requesting a permission set for a specific user by ID SHOULD work with authentication.
    // TODO

    // Attempting to update a non-existent user's display name MUST fail.
    // TODO

    // Updating another user's display name MUST fail without authentication.
    // TODO

    // Updating another user's display name (with authentication) to an existing name MUST fail.
    // TODO

    // Updating another user's display name (with authentication) MUST fail if the caller does not have enough permissions.
    // TODO

    // Updating another user's display name (with authentication) MUST work if the caller has enough permissions.
    // TODO

    // Updating your own user's display name (with authentication) through a non-"/me/*" endpoint MUST fail.
    // TODO

    // Adding a role to non-existent user's role set MUST fail.
    // TODO

    // Adding a role to another user's role set MUST fail without authentication.
    // TODO

    // Adding a role to another user's role set (with authentication) MUST fail if the caller does not have all the implied permissions they want to add to that user.
    // TODO

    // Adding a role to another user's role set (with authentication) MUST fail if any role name is invalid.
    // TODO

    // Adding a role to another user's role set (with authentication) MUST succeed if the caller has all the implied permissions they want to add to that user.
    // TODO

    // Adding a role to your own user's role set (with authentication) through the generic user role update endpoint MUST fail.
    // TODO

    // Removing a role from non-existent user's role set MUST fail.
    // TODO

    // Removing a role from another user's role set MUST fail without authentication.
    // TODO

    // Removing a role from another user's role set (with authentication) MUST fail if the caller does not have all the implied permissions they want to remove from that user.
    // TODO

    // Removing a role from another user's role set (with authentication) MUST fail if any role name is invalid.
    // TODO

    // Removing a role from another user's role set (with authentication) MUST succeed if the caller has all the implied permissions they want to remove from that user.
    // TODO

    // Removing a role from your own user's role set (with authentication) through the generic user role update endpoint MUST fail.
    // TODO


    todo!();
}
