use chrono::Utc;
use kolomoni_core::ids::EnglishWordId;
use sqlx::PgConnection;

use crate::{
    entities::{
        word::{NewWord, WordLanguage, WordMutation},
        word_english::internal_insert_only::InsertOnlyInternalEnglishWordModel,
    },
    QueryError,
    QueryResult,
};



#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NewEnglishWord {
    pub lemma: String,
}


#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EnglishWordFieldsToUpdate {
    /// Because we might introduce other updatable fields in the future,
    /// this field is an [`Option`] (`None` indicates you do not wish to modify the lemma).
    pub new_lemma: Option<String>,
}



pub struct EnglishWordMutation;

impl EnglishWordMutation {
    pub async fn create(
        database_connection: &mut PgConnection,
        word_to_create: NewEnglishWord,
    ) -> QueryResult<super::EnglishWordModel> {
        let new_word_created_at = Utc::now();
        let new_word_last_modified_at = new_word_created_at;


        let new_word = WordMutation::create(
            database_connection,
            NewWord {
                language: WordLanguage::English,
                created_at: new_word_created_at,
                last_modified_at: new_word_last_modified_at,
            },
        )
        .await?;

        let english_word_model = sqlx::query_as!(
            InsertOnlyInternalEnglishWordModel,
            "INSERT INTO kolomoni.word_english \
                    (word_id, lemma) \
                VALUES \
                    ($1, $2) \
                RETURNING \
                    word_id, lemma",
            new_word.id.into_uuid(),
            &word_to_create.lemma,
        )
        .fetch_one(database_connection)
        .await?;


        Ok(super::EnglishWordModel::new_with_lemma(
            new_word,
            english_word_model.lemma,
        ))
    }

    pub async fn update(
        database_connection: &mut PgConnection,
        english_word_id: EnglishWordId,
        fields_to_update: EnglishWordFieldsToUpdate,
    ) -> QueryResult<bool> {
        let Some(new_lemma) = fields_to_update.new_lemma else {
            return Ok(true);
        };


        let query_result = sqlx::query!(
            "UPDATE kolomoni.word_english \
                SET lemma = $1 \
                WHERE word_id = $2",
            new_lemma,
            english_word_id.into_uuid()
        )
        .execute(database_connection)
        .await?;

        if query_result.rows_affected() > 1 {
            return Err(QueryError::database_inconsistency(
                "more than one row was affected when updating an english word",
            ));
        }

        Ok(query_result.rows_affected() == 1)
    }

    pub async fn delete(
        database_connection: &mut PgConnection,
        english_word_id: EnglishWordId,
    ) -> QueryResult<bool> {
        // This is the only required SQL query, because the associated
        // english word table (and by proxy, associated meanings), will be
        // automatically deleted due to `ON DELETE CASCADE`.
        WordMutation::delete(
            database_connection,
            english_word_id.upcast_to_word_id(),
        )
        .await
    }
}
