use kolomoni::api::v1::dictionary::{
    english_word::{
        EnglishWord,
        EnglishWordCreationRequest,
        EnglishWordCreationResponse,
        EnglishWordInfoResponse,
        EnglishWordsResponse,
    },
    slovene_word::{
        SloveneWord,
        SloveneWordCreationRequest,
        SloveneWordCreationResponse,
        SloveneWordsResponse,
    },
    suggestions::{TranslationSuggestionDeletionRequest, TranslationSuggestionRequest},
    translations::{TranslationDeletionRequest, TranslationRequest},
};
use kolomoni_test_util::prelude::*;



#[tokio::test]
async fn word_creation_with_suggestions_and_translations_works() {
    let server = prepare_test_server_instance().await;

    register_sample_user(&server, SampleUser::Janez).await;
    register_sample_user(&server, SampleUser::Meta).await;

    let admin_user_access_token = login_sample_user(&server, SampleUser::Janez).await;
    let admin_user_info = get_sample_user_info(&server, &admin_user_access_token).await;

    let normal_user_access_token = login_sample_user(&server, SampleUser::Meta).await;

    server
        .give_full_permissions_to_user(admin_user_info.id)
        .await;


    /***
     * Test english word listing, creation and deletion.
     */

    {
        // Unauthenticated users should be able to list english words.
        server
            .request(Method::GET, "/api/v1/dictionary/english")
            .send()
            .await
            .assert_status_equals(StatusCode::OK);

        // Normal users should be able to list english words.
        server
            .request(Method::GET, "/api/v1/dictionary/english")
            .with_access_token(&normal_user_access_token)
            .send()
            .await
            .assert_status_equals(StatusCode::OK);


        // Total number of english words should be 0 on a fresh database.
        let query_response = server
            .request(Method::GET, "/api/v1/dictionary/english")
            .with_access_token(&admin_user_access_token)
            .send()
            .await;

        query_response.assert_status_equals(StatusCode::OK);

        let english_word_list = query_response
            .json_body::<EnglishWordsResponse>()
            .english_words;
        assert_eq!(english_word_list.len(), 0);
    }


    {
        // If no JSON body is provided, the request should fail with 400 Bad Request.
        server
            .request(Method::POST, "/api/v1/dictionary/english")
            .send()
            .await
            .assert_status_equals(StatusCode::BAD_REQUEST);

        // Authorization should be required.
        server
            .request(Method::POST, "/api/v1/dictionary/english")
            .with_json_body(EnglishWordCreationRequest {
                lemma: "test".to_string(),
                disambiguation: Some("test".to_string()),
                description: Some("test".to_string()),
            })
            .send()
            .await
            .assert_status_equals(StatusCode::UNAUTHORIZED);

        // Normal users shouldn't be able to create new english words.
        server
            .request(Method::POST, "/api/v1/dictionary/english")
            .with_json_body(EnglishWordCreationRequest {
                lemma: "test".to_string(),
                disambiguation: Some("test".to_string()),
                description: Some("test".to_string()),
            })
            .with_access_token(&normal_user_access_token)
            .send()
            .await
            .assert_status_equals(StatusCode::FORBIDDEN);
    }

    let new_english_test_word = {
        let creation_response = server
            .request(Method::POST, "/api/v1/dictionary/english")
            .with_json_body(EnglishWordCreationRequest {
                lemma: "test".to_string(),
                disambiguation: Some("test".to_string()),
                description: Some("test".to_string()),
            })
            .with_access_token(&admin_user_access_token)
            .send()
            .await;

        creation_response.assert_status_equals(StatusCode::OK);

        creation_response
            .json_body::<EnglishWordCreationResponse>()
            .word
    };


    {
        // Total number of english words should have increased to 1
        // and the entry should match the word we just inserted.
        let query_response = server
            .request(Method::GET, "/api/v1/dictionary/english")
            .with_access_token(&admin_user_access_token)
            .send()
            .await;

        query_response.assert_status_equals(StatusCode::OK);

        let english_word_list = query_response
            .json_body::<EnglishWordsResponse>()
            .english_words;

        assert_eq!(english_word_list.len(), 1);

        let sample_word = &english_word_list[0];
        assert_eq!(sample_word, &new_english_test_word);
    }


    {
        // Deleting an english word should require authentication.
        server
            .request(
                Method::DELETE,
                format!(
                    "/api/v1/dictionary/english/{}",
                    new_english_test_word.word_id
                ),
            )
            .send()
            .await
            .assert_status_equals(StatusCode::UNAUTHORIZED);

        // Sending a request with an invalid UUID should fail.
        server
            .request(
                Method::DELETE,
                "/api/v1/dictionary/english/293875djkfsq",
            )
            .with_access_token(&admin_user_access_token)
            .send()
            .await
            .assert_status_equals(StatusCode::BAD_REQUEST);

        // Normal users should not be able to delete english words.
        server
            .request(
                Method::DELETE,
                format!(
                    "/api/v1/dictionary/english/{}",
                    new_english_test_word.word_id
                ),
            )
            .with_access_token(&normal_user_access_token)
            .send()
            .await
            .assert_status_equals(StatusCode::FORBIDDEN);

        // Trying to delete a non-existent english word should fail with 404.
        server
            .request(
                Method::DELETE,
                "/api/v1/dictionary/english/018dcd50-8e5f-7e1e-8437-60898a3dc18c",
            )
            .with_access_token(&admin_user_access_token)
            .send()
            .await
            .assert_status_equals(StatusCode::NOT_FOUND);
    }

    {
        server
            .request(
                Method::DELETE,
                format!(
                    "/api/v1/dictionary/english/{}",
                    new_english_test_word.word_id
                ),
            )
            .with_access_token(&admin_user_access_token)
            .send()
            .await
            .assert_status_equals(StatusCode::OK);
    }


    {
        // Total number of english words should decrease to 0.
        let query_response = server
            .request(Method::GET, "/api/v1/dictionary/english")
            .with_access_token(&admin_user_access_token)
            .send()
            .await;

        query_response.assert_status_equals(StatusCode::OK);

        let english_word_list = query_response
            .json_body::<EnglishWordsResponse>()
            .english_words;
        assert_eq!(english_word_list.len(), 0);
    }




    /***
     * Test slovene word listing, creation and deletion
     */

    {
        // Unauthenticated users should be able to list slovene words.
        server
            .request(Method::GET, "/api/v1/dictionary/slovene")
            .send()
            .await
            .assert_status_equals(StatusCode::OK);

        // Normal users should be able to list slovene words.
        server
            .request(Method::GET, "/api/v1/dictionary/slovene")
            .with_access_token(&normal_user_access_token)
            .send()
            .await
            .assert_status_equals(StatusCode::OK);


        // Total number of slovene words should be 0 on a fresh database.
        let query_response = server
            .request(Method::GET, "/api/v1/dictionary/slovene")
            .with_access_token(&admin_user_access_token)
            .send()
            .await;

        query_response.assert_status_equals(StatusCode::OK);

        let slovene_word_list = query_response
            .json_body::<SloveneWordsResponse>()
            .slovene_words;
        assert_eq!(slovene_word_list.len(), 0);
    }


    {
        // If no JSON body is provided, the request should fail with 400 Bad Request.
        server
            .request(Method::POST, "/api/v1/dictionary/slovene")
            .send()
            .await
            .assert_status_equals(StatusCode::BAD_REQUEST);

        // Authorization should be required.
        server
            .request(Method::POST, "/api/v1/dictionary/slovene")
            .with_json_body(SloveneWordCreationRequest {
                lemma: "test".to_string(),
                disambiguation: Some("test".to_string()),
                description: Some("test".to_string()),
            })
            .send()
            .await
            .assert_status_equals(StatusCode::UNAUTHORIZED);

        // Normal users shouldn't be able to create new slovene words.
        server
            .request(Method::POST, "/api/v1/dictionary/slovene")
            .with_json_body(SloveneWordCreationRequest {
                lemma: "test".to_string(),
                disambiguation: Some("test".to_string()),
                description: Some("test".to_string()),
            })
            .with_access_token(&normal_user_access_token)
            .send()
            .await
            .assert_status_equals(StatusCode::FORBIDDEN);
    }

    let new_slovene_test_word = {
        let creation_response = server
            .request(Method::POST, "/api/v1/dictionary/slovene")
            .with_json_body(SloveneWordCreationRequest {
                lemma: "test".to_string(),
                disambiguation: Some("test".to_string()),
                description: Some("test".to_string()),
            })
            .with_access_token(&admin_user_access_token)
            .send()
            .await;

        creation_response.assert_status_equals(StatusCode::OK);

        creation_response
            .json_body::<SloveneWordCreationResponse>()
            .word
    };


    {
        // Total number of slovene words should have increased to 1
        // and the entry should match the word we just inserted.
        let query_response = server
            .request(Method::GET, "/api/v1/dictionary/slovene")
            .with_access_token(&admin_user_access_token)
            .send()
            .await;

        query_response.assert_status_equals(StatusCode::OK);

        let slovene_word_list = query_response
            .json_body::<SloveneWordsResponse>()
            .slovene_words;
        assert_eq!(slovene_word_list.len(), 1);

        let sample_word = &slovene_word_list[0];
        assert_eq!(sample_word, &new_slovene_test_word);
    }


    {
        // Deleting a slovene word should require authentication.
        server
            .request(
                Method::DELETE,
                format!(
                    "/api/v1/dictionary/slovene/{}",
                    new_slovene_test_word.word_id
                ),
            )
            .send()
            .await
            .assert_status_equals(StatusCode::UNAUTHORIZED);

        // Sending a request with an invalid UUID should fail.
        server
            .request(
                Method::DELETE,
                "/api/v1/dictionary/slovene/293875djkfsq",
            )
            .with_access_token(&admin_user_access_token)
            .send()
            .await
            .assert_status_equals(StatusCode::BAD_REQUEST);

        // Normal users should not be able to delete slovene words.
        server
            .request(
                Method::DELETE,
                format!(
                    "/api/v1/dictionary/slovene/{}",
                    new_slovene_test_word.word_id
                ),
            )
            .with_access_token(&normal_user_access_token)
            .send()
            .await
            .assert_status_equals(StatusCode::FORBIDDEN);

        // Trying to delete a non-existent slovene word should fail with 404.
        server
            .request(
                Method::DELETE,
                "/api/v1/dictionary/slovene/018dcd50-8e5f-7e1e-8437-60898a3dc18c",
            )
            .with_access_token(&admin_user_access_token)
            .send()
            .await
            .assert_status_equals(StatusCode::NOT_FOUND);
    }

    {
        server
            .request(
                Method::DELETE,
                format!(
                    "/api/v1/dictionary/slovene/{}",
                    new_slovene_test_word.word_id
                ),
            )
            .with_access_token(&admin_user_access_token)
            .send()
            .await
            .assert_status_equals(StatusCode::OK);
    }


    {
        // Total number of slovene words should have decreased to 0.
        let query_response = server
            .request(Method::GET, "/api/v1/dictionary/slovene")
            .with_access_token(&admin_user_access_token)
            .send()
            .await;

        query_response.assert_status_equals(StatusCode::OK);

        let slovene_word_list = query_response
            .json_body::<SloveneWordsResponse>()
            .slovene_words;
        assert_eq!(slovene_word_list.len(), 0);
    }



    /***
     * Insert a bunch of sample words to test on.
     */

    let word_ability = create_sample_english_word(
        &server,
        &admin_user_access_token,
        SampleEnglishWord::Ability,
    )
    .await;
    let word_critical_hit = create_sample_english_word(
        &server,
        &admin_user_access_token,
        SampleEnglishWord::CriticalHit,
    )
    .await;

    let word_sposobnost = create_sample_slovene_word(
        &server,
        &admin_user_access_token,
        SampleSloveneWord::Sposobnost,
    )
    .await;
    let word_terna = create_sample_slovene_word(
        &server,
        &admin_user_access_token,
        SampleSloveneWord::Terna,
    )
    .await;
    let word_kriticni_izid = create_sample_slovene_word(
        &server,
        &admin_user_access_token,
        SampleSloveneWord::KriticniIzid,
    )
    .await;
    let word_usodni_zadetek = create_sample_slovene_word(
        &server,
        &admin_user_access_token,
        SampleSloveneWord::UsodniZadetek,
    )
    .await;




    /***
     * Test translation suggestions (creation, querying and deletion).
     */

    {
        // A valid JSON body must be provided.
        server
            .request(Method::POST, "/api/v1/dictionary/suggestion")
            .send()
            .await
            .assert_status_equals(StatusCode::BAD_REQUEST);

        // Creating a suggestion should require authentication.
        server
            .request(Method::POST, "/api/v1/dictionary/suggestion")
            .with_json_body(TranslationSuggestionRequest {
                english_word_id: word_ability.word_id.to_string(),
                slovene_word_id: word_sposobnost.word_id.to_string(),
            })
            .send()
            .await
            .assert_status_equals(StatusCode::UNAUTHORIZED);

        // The request should fail if any UUIDs are invalid.
        server
            .request(Method::POST, "/api/v1/dictionary/suggestion")
            .with_json_body(TranslationSuggestionRequest {
                english_word_id: "asdo214sdaf".to_string(),
                slovene_word_id: word_sposobnost.word_id.to_string(),
            })
            .with_access_token(&admin_user_access_token)
            .send()
            .await
            .assert_status_equals(StatusCode::BAD_REQUEST);

        // The request should fail if any UUIDs don't exist.
        server
            .request(Method::POST, "/api/v1/dictionary/suggestion")
            .with_json_body(TranslationSuggestionRequest {
                english_word_id: "018dcd50-8e5f-7e1e-8437-60898a3dc18c".to_string(),
                slovene_word_id: word_sposobnost.word_id.to_string(),
            })
            .with_access_token(&admin_user_access_token)
            .send()
            .await
            .assert_status_equals(StatusCode::BAD_REQUEST);
    }

    {
        let suggestion_response = server
            .request(Method::POST, "/api/v1/dictionary/suggestion")
            .with_json_body(TranslationSuggestionRequest {
                english_word_id: word_ability.word_id.to_string(),
                slovene_word_id: word_sposobnost.word_id.to_string(),
            })
            .with_access_token(&admin_user_access_token)
            .send()
            .await;

        suggestion_response.assert_status_equals(StatusCode::OK);



        let ability_word_response = server
            .request(
                Method::GET,
                format!(
                    "/api/v1/dictionary/english/{}",
                    word_ability.word_id
                ),
            )
            .send()
            .await;

        ability_word_response.assert_status_equals(StatusCode::OK);
        let ability_word_information = ability_word_response
            .json_body::<EnglishWordInfoResponse>()
            .word;

        assert_eq!(
            ability_word_information.suggested_translations.len(),
            1
        );
        assert_eq!(
            &ability_word_information.suggested_translations[0],
            &word_sposobnost
        );



        // Trying to create the same suggestion again should fail with 409 Conflict.
        server
            .request(Method::POST, "/api/v1/dictionary/suggestion")
            .with_json_body(TranslationSuggestionRequest {
                english_word_id: word_ability.word_id.to_string(),
                slovene_word_id: word_sposobnost.word_id.to_string(),
            })
            .with_access_token(&admin_user_access_token)
            .send()
            .await
            .assert_status_equals(StatusCode::CONFLICT);
    }


    {
        // Failing to provide a JSON body should result in 400 Bad Request.
        server
            .request(Method::DELETE, "/api/v1/dictionary/suggestion")
            .send()
            .await
            .assert_status_equals(StatusCode::BAD_REQUEST);

        // Deleting a suggestion should require authentication.
        server
            .request(Method::DELETE, "/api/v1/dictionary/suggestion")
            .with_json_body(TranslationSuggestionDeletionRequest {
                english_word_id: word_critical_hit.word_id.to_string(),
                slovene_word_id: word_usodni_zadetek.word_id.to_string(),
            })
            .send()
            .await
            .assert_status_equals(StatusCode::UNAUTHORIZED);


        // Trying to delete a non-existent suggestion should fail with 404 Not Found.
        server
            .request(Method::DELETE, "/api/v1/dictionary/suggestion")
            .with_access_token(&admin_user_access_token)
            .with_json_body(TranslationSuggestionDeletionRequest {
                english_word_id: word_critical_hit.word_id.to_string(),
                slovene_word_id: word_usodni_zadetek.word_id.to_string(),
            })
            .send()
            .await
            .assert_status_equals(StatusCode::NOT_FOUND);
    }

    {
        let suggestion_removal_response = server
            .request(Method::DELETE, "/api/v1/dictionary/suggestion")
            .with_access_token(&admin_user_access_token)
            .with_json_body(TranslationSuggestionDeletionRequest {
                english_word_id: word_ability.word_id.to_string(),
                slovene_word_id: word_sposobnost.word_id.to_string(),
            })
            .send()
            .await;

        suggestion_removal_response.assert_status_equals(StatusCode::OK);
    }

    {
        let ability_word_response = server
            .request(
                Method::GET,
                format!(
                    "/api/v1/dictionary/english/{}",
                    word_ability.word_id
                ),
            )
            .send()
            .await;

        ability_word_response.assert_status_equals(StatusCode::OK);
        let ability_word_information = ability_word_response
            .json_body::<EnglishWordInfoResponse>()
            .word;

        assert_eq!(
            ability_word_information.suggested_translations.len(),
            0
        );
    }



    /***
     * Test translations (creation, querying and deletion).
     */

    {
        // A valid JSON body must be provided.
        server
            .request(Method::POST, "/api/v1/dictionary/translation")
            .send()
            .await
            .assert_status_equals(StatusCode::BAD_REQUEST);

        // Creating a suggestion should require authentication.
        server
            .request(Method::POST, "/api/v1/dictionary/translation")
            .with_json_body(TranslationRequest {
                english_word_id: word_ability.word_id.to_string(),
                slovene_word_id: word_sposobnost.word_id.to_string(),
            })
            .send()
            .await
            .assert_status_equals(StatusCode::UNAUTHORIZED);

        // The request should fail if any UUIDs are invalid.
        server
            .request(Method::POST, "/api/v1/dictionary/translation")
            .with_json_body(TranslationRequest {
                english_word_id: "asdo214sdaf".to_string(),
                slovene_word_id: word_sposobnost.word_id.to_string(),
            })
            .with_access_token(&admin_user_access_token)
            .send()
            .await
            .assert_status_equals(StatusCode::BAD_REQUEST);

        // The request should fail if any UUIDs don't exist.
        server
            .request(Method::POST, "/api/v1/dictionary/translation")
            .with_json_body(TranslationRequest {
                english_word_id: "018dcd50-8e5f-7e1e-8437-60898a3dc18c".to_string(),
                slovene_word_id: word_sposobnost.word_id.to_string(),
            })
            .with_access_token(&admin_user_access_token)
            .send()
            .await
            .assert_status_equals(StatusCode::BAD_REQUEST);
    }

    {
        let translation_response = server
            .request(Method::POST, "/api/v1/dictionary/translation")
            .with_json_body(TranslationRequest {
                english_word_id: word_ability.word_id.to_string(),
                slovene_word_id: word_sposobnost.word_id.to_string(),
            })
            .with_access_token(&admin_user_access_token)
            .send()
            .await;

        translation_response.assert_status_equals(StatusCode::OK);



        let ability_word_response = server
            .request(
                Method::GET,
                format!(
                    "/api/v1/dictionary/english/{}",
                    word_ability.word_id
                ),
            )
            .send()
            .await;

        ability_word_response.assert_status_equals(StatusCode::OK);
        let ability_word_information = ability_word_response
            .json_body::<EnglishWordInfoResponse>()
            .word;

        assert_eq!(ability_word_information.translations.len(), 1);
        assert_eq!(
            &ability_word_information.translations[0],
            &word_sposobnost
        );



        // Trying to create the same translation again should fail with 409 Conflict.
        server
            .request(Method::POST, "/api/v1/dictionary/translation")
            .with_json_body(TranslationRequest {
                english_word_id: word_ability.word_id.to_string(),
                slovene_word_id: word_sposobnost.word_id.to_string(),
            })
            .with_access_token(&admin_user_access_token)
            .send()
            .await
            .assert_status_equals(StatusCode::CONFLICT);
    }

    {
        // Failing to provide a JSON body should result in 400 Bad Request.
        server
            .request(Method::DELETE, "/api/v1/dictionary/translation")
            .with_access_token(&admin_user_access_token)
            .send()
            .await
            .assert_status_equals(StatusCode::BAD_REQUEST);

        // Deleting a translation should require authentication.
        server
            .request(Method::DELETE, "/api/v1/dictionary/translation")
            .with_json_body(TranslationDeletionRequest {
                english_word_id: word_critical_hit.word_id.to_string(),
                slovene_word_id: word_usodni_zadetek.word_id.to_string(),
            })
            .send()
            .await
            .assert_status_equals(StatusCode::UNAUTHORIZED);

        // Trying to delete a non-existent suggestion should fail with 404 Not Found.
        server
            .request(Method::DELETE, "/api/v1/dictionary/translation")
            .with_access_token(&admin_user_access_token)
            .with_json_body(TranslationDeletionRequest {
                english_word_id: word_critical_hit.word_id.to_string(),
                slovene_word_id: word_usodni_zadetek.word_id.to_string(),
            })
            .send()
            .await
            .assert_status_equals(StatusCode::NOT_FOUND);
    }

    {
        let translation_removal_response = server
            .request(Method::DELETE, "/api/v1/dictionary/translation")
            .with_access_token(&admin_user_access_token)
            .with_json_body(TranslationDeletionRequest {
                english_word_id: word_ability.word_id.to_string(),
                slovene_word_id: word_sposobnost.word_id.to_string(),
            })
            .send()
            .await;

        translation_removal_response.assert_status_equals(StatusCode::OK);
    }

    {
        let ability_word_response = server
            .request(
                Method::GET,
                format!(
                    "/api/v1/dictionary/english/{}",
                    word_ability.word_id
                ),
            )
            .send()
            .await;

        ability_word_response.assert_status_equals(StatusCode::OK);
        let ability_word_information = ability_word_response
            .json_body::<EnglishWordInfoResponse>()
            .word;

        assert_eq!(ability_word_information.translations.len(), 0);
    }



    /***
     * Prep some more complex situation and then check that things work as expected.
     */

    link_word_as_translation(
        &server,
        &admin_user_access_token,
        &word_ability.word_id,
        &word_sposobnost.word_id,
    )
    .await;

    link_word_as_suggested_translation(
        &server,
        &admin_user_access_token,
        &word_critical_hit.word_id,
        &word_terna.word_id,
    )
    .await;
    link_word_as_suggested_translation(
        &server,
        &admin_user_access_token,
        &word_critical_hit.word_id,
        &word_kriticni_izid.word_id,
    )
    .await;
    link_word_as_suggested_translation(
        &server,
        &admin_user_access_token,
        &word_critical_hit.word_id,
        &word_usodni_zadetek.word_id,
    )
    .await;



    let english_word_list = {
        let query_response = server
            .request(Method::GET, "/api/v1/dictionary/english")
            .with_access_token(&admin_user_access_token)
            .send()
            .await;

        query_response.assert_status_equals(StatusCode::OK);

        let english_word_list = query_response
            .json_body::<EnglishWordsResponse>()
            .english_words;
        assert_eq!(english_word_list.len(), 2);

        assert!(english_word_list
            .iter()
            .any(|word| word.word_id == word_ability.word_id));
        assert!(english_word_list
            .iter()
            .any(|word| word.word_id == word_critical_hit.word_id));

        english_word_list
    };

    {
        let query_response = server
            .request(Method::GET, "/api/v1/dictionary/slovene")
            .with_access_token(&admin_user_access_token)
            .send()
            .await;

        query_response.assert_status_equals(StatusCode::OK);

        let slovene_word_list = query_response
            .json_body::<SloveneWordsResponse>()
            .slovene_words;
        assert_eq!(slovene_word_list.len(), 4);

        assert!(slovene_word_list
            .iter()
            .any(|word| word.word_id == word_sposobnost.word_id));
        assert!(slovene_word_list
            .iter()
            .any(|word| word.word_id == word_terna.word_id));
        assert!(slovene_word_list
            .iter()
            .any(|word| word.word_id == word_kriticni_izid.word_id));
        assert!(slovene_word_list
            .iter()
            .any(|word| word.word_id == word_usodni_zadetek.word_id));
    }


    // Check that "ability" is translated into "sposobnost".
    {
        let queried_word_ability = english_word_list
            .iter()
            .find(|word| word.word_id == word_ability.word_id)
            .unwrap();

        assert_eq!(
            queried_word_ability.suggested_translations.len(),
            0
        );
        assert_eq!(queried_word_ability.translations.len(), 1);

        let ability_translation = &queried_word_ability.translations[0];
        assert_eq!(ability_translation, &word_sposobnost);
    }


    // Check that "critical hit" has three suggested translations: "terna", "kritični izid" and "usodni zadetek".
    {
        let queried_word_critical_hit = english_word_list.iter()
        .find(|word| word.word_id == word_critical_hit.word_id).unwrap();

        assert_eq!(queried_word_critical_hit.suggested_translations.len(), 3);
        assert_eq!(queried_word_critical_hit.translations.len(), 0);


        let translated_terna = queried_word_critical_hit.suggested_translations.iter()
            .find(|suggestion| suggestion.word_id == word_terna.word_id).unwrap();
        assert_eq!(
            translated_terna,
            &word_terna
        );


        let translated_kriticni_izid = queried_word_critical_hit.suggested_translations.iter()
            .find(|suggestion| suggestion.word_id == word_kriticni_izid.word_id).unwrap();
        assert_eq!(
            translated_kriticni_izid,
            &word_kriticni_izid
        );


        let translated_usodni_zadetek = queried_word_critical_hit.suggested_translations.iter()
            .find(|suggestion| suggestion.word_id == word_usodni_zadetek.word_id).unwrap();
        assert_eq!(
            translated_usodni_zadetek,
            &word_usodni_zadetek
        );
    }
}
