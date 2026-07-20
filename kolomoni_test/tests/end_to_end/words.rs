use kolomoni_test_core::{fresh_test_client, macros::test as kolomoni_test};

#[kolomoni_test]
async fn asd() {
    let client = fresh_test_client!();

    // TODO test english word listing
    //      (unauthenticated users should be able to list english words)
    //      (normal authenticated users should be able to list english words)
    //      (total number of words should be zero in an empty database)

    // TODO test english word creation and update
    //      (authentication must be required and sufficient permissions)
    //      (normal users must not be able to create new english words)
    //      (total number of listed words must increase when creating the word)

    // TODO test english word deletion
    //      (authentication must be required and sufficient permissions)
    //      (deleting a non-existent english word must fail)
    //      (normal users must not be able to delete english words)
    //      (total number of listed words must decrease when deleting the word)

    // TODO test english word meaning creation and update
    // TODO test english word meaning querying
    // TODO test english word meaning querying with filters
    //      (test filter by modification time)
    // TODO test english word meaning deletion

    // TODO test slovene word listing
    //      (unauthenticated users should be able to list slovene words)
    //      (normal authenticated users should be able to list english words)
    //      (total number of words should be zero in an empty database)

    // TODO test slovene word creation and update
    //      (authentication must be required and sufficient permissions)
    //      (normal users must not be able to create new slovene words)
    //      (total number of listed words must increase when creating the word)

    // TODO test slovene word deletion
    //      (authentication must be required and sufficient permissions)
    //      (deleting a non-existent slovene word must fail)
    //      (normal users must not be able to delete slovene words)
    //      (total number of listed words must decrease when deleting the word)

    // TODO test slovene word meaning creation and update
    // TODO test slovene word meaning querying
    // TODO test slovene word meaning querying with filters
    //      (test filter by modification time)
    // TODO test slovene word meaning deletion

    // TODO test translation creation (and update?)
    //      (creating a translation must require authentication and sufficient permissions)
    //      (last_edited_at values must change on both words when a translation is applied)
    //      (trying to create the same translation again must fail as a conflict)

    // TODO test translation querying

    // TODO test translation deletion
    //      (deleting a translation must require authentication and sufficient permissions)
    //      (trying to delete a non-existent translation must fail)
    //      (last_edited_at values must change on both words when a translation is deleted)

    // TODO prepare small examples and test their validity (especially consider multiple translations on one word meaning)

    // TODO lookup by english lemma must work without authentication
    // TODO lookup by english lemma must fail when no such word exists

    // TODO lookup by slovene lemma must work without authentication
    // TODO lookup by slovene lemma must fail when no such word exists

    // TODO test category creation
    //      (category creation must require authentication and sufficient permissions)
    //      (creating a category with an existing name must fail)
    // TODO test category update
    //      (must fail if there is no such category)
    //      (must require authentication and sufficient permissions)
    //      (must fail if attempting to change into an existing name)
    // TODO test category query
    //      (category query must not require authentication)
    // TODO test category deletion
    //      (must fail if there is no such category)
    //      (must require authentication and sufficient permissions)

    // TODO test category to english word meaning link
    //      (must fail if there is no such category)
    //      (must fail if there is no such word meaning)
    //      (must require authentication and sufficient permissions)
    // TODO test category to english word meaning unlink
    //      (must fail if there is no such category)
    //      (must fail if there is no such word meaning)
    //      (must require authentication and sufficient permissions)
    // TODO test category to slovene word meaning link
    //      (must fail if there is no such category)
    //      (must fail if there is no such word meaning)
    //      (must require authentication and sufficient permissions)
    // TODO test category to slovene word meaning unlink
    //      (must fail if there is no such category)
    //      (must fail if there is no such word meaning)
    //      (must require authentication and sufficient permissions)


    todo!();
}
