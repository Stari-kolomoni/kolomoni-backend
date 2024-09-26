use chrono::Utc;
use kolomoni_core::id::EnglishWordId;
use sqlx::PgConnection;

use crate::{
    entities::{self, WordLanguage, WordMutation},
    QueryError,
    QueryResult,
};


#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NewEnglishWord {
    pub lemma: String,
}


#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EnglishWordFieldsToUpdate {
    pub new_lemma: Option<String>,
}



pub struct EnglishWordMutation;

impl EnglishWordMutation {
    pub async fn create(
        database_connection: &mut PgConnection,
        word_to_create: NewEnglishWord,
    ) -> QueryResult<super::EnglishWordModel> {
        let new_word_id = EnglishWordId::generate();
        let new_word_language_code = WordLanguage::English.to_ietf_bcp_47_language_tag();
        let new_word_created_at = Utc::now();
        let new_word_last_modified_at = new_word_created_at;

        let bare_word_model = sqlx::query_as!(
            entities::InternalWordModel,
            "INSERT INTO kolomoni.word (id, language_code, created_at, last_modified_at) \
                VALUES ($1, $2, $3, $4) \
                RETURNING id, language_code, created_at, last_modified_at",
            new_word_id.into_uuid(),
            new_word_language_code,
            new_word_created_at,
            new_word_last_modified_at
        )
        .fetch_one(&mut *database_connection)
        .await?;

        let english_word_model = sqlx::query_as!(
            super::InternalEnglishWordReducedModel,
            "INSERT INTO kolomoni.word_english (word_id, lemma) \
                VALUES ($1, $2) \
                RETURNING word_id, lemma",
            new_word_id.into_uuid(),
            &word_to_create.lemma,
        )
        .fetch_one(database_connection)
        .await?;


        Ok(super::EnglishWordModel {
            word_id: EnglishWordId::new(english_word_model.word_id),
            lemma: english_word_model.lemma,
            created_at: bare_word_model.created_at,
            last_modified_at: bare_word_model.last_modified_at,
        })
    }

    pub async fn update(
        database_connection: &mut PgConnection,
        english_word_id: EnglishWordId,
        fields_to_update: EnglishWordFieldsToUpdate,
    ) -> QueryResult<bool> {
        let Some(new_lemma) = fields_to_update.new_lemma else {
            return Ok(true);
        };


        let english_word_id = english_word_id.into_uuid();

        let query_result = sqlx::query!(
            "UPDATE kolomoni.word_english \
                SET lemma = $1 \
                WHERE word_id = $2",
            new_lemma,
            english_word_id
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
        WordMutation::delete(
            database_connection,
            english_word_id.into_word_id(),
        )
        .await
    }
}
