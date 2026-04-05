use chrono::Utc;
use kolomoni_api_client::{
    api::auth::{UserRegistrationError, UserRegistrationInfo},
    authentication::ServerAuthentication,
    SharedApiClientEndpointGroups,
};
use kolomoni_test_util::{
    ensure_request_is_ok,
    initialize_fresh_test_server,
    macros::assert_matches,
    prelude::{SampleUser, TestServerClient},
};


#[tokio::test]
async fn user_registration_and_listing_works() {
    let client = initialize_fresh_test_server().await;

    // Register a new user, then manually fetch the user and ensure they are the same.
    let time_before_registration = Utc::now();
    let new_user = SampleUser::Janez.register(&client).await;
    let time_after_registration = Utc::now();

    let new_fetched_user = ensure_request_is_ok!(
        client
            .users()
            .get_user_information_by_user_id(new_user.id)
            .await,
        "failed to obtain user information of the just-registered sample user"
    );

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

    let server_authentication =
        ServerAuthentication::new_from_token_set(SampleUser::Janez.login(&client).await);
    let auhenticated_client = client.with_authentication(server_authentication);


    // Ensure the registered user appears in the full user list.
    {
        let all_registered_users = ensure_request_is_ok!(
            auhenticated_client.users().get_all_registered_users().await,
            "failed to fetch all registered users"
        );

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

        let all_registered_users = ensure_request_is_ok!(
            auhenticated_client.users().get_all_registered_users().await,
            "failed to fetch all registered user again"
        );

        assert!(all_registered_users.len() == 2);
    }


    // Registering with an existing username should fail.
    let re_registration_error = client
        .authentication()
        .register_user(UserRegistrationInfo {
            username: SampleUser::Janez.username().to_owned(),
            display_name: "SAMPLE".to_owned(),
            password: SampleUser::Janez.password().to_owned(),
        })
        .await
        .unwrap_err();

    assert_matches!(
        re_registration_error,
        UserRegistrationError::UsernameAlreadyExists
    );


    // Registering with an existing display name should fail.
    let re_registration_error = client
        .authentication()
        .register_user(UserRegistrationInfo {
            username: "SAMPLE".to_owned(),
            display_name: SampleUser::Janez.display_name().to_owned(),
            password: SampleUser::Janez.password().to_owned(),
        })
        .await
        .unwrap_err();

    assert_matches!(
        re_registration_error,
        UserRegistrationError::DisplayNameAlreadyExists
    );
}



#[tokio::test]
async fn listing_user_permissions_and_roles_works() {
    let client = initialize_fresh_test_server().await;


    SampleUser::Janez.register(&client).await;
    SampleUser::Meta.register(&client).await;

    let server_authentication =
        ServerAuthentication::new_from_token_set(SampleUser::Janez.login(&client).await);
    let authenticated_client = client.with_authentication(server_authentication);


    let janez_roles = ensure_request_is_ok!(
        authenticated_client
            .current_user()
            .get_current_user_roles()
            .await,
        "failed to get current user roles"
    );

    // TODO continue from here (we need to test 403 responses for secured endpoints as well, think about that)

    todo!();
}
