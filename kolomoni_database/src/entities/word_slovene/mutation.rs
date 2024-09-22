use chrono::Utc;
use kolomoni_core::id::SloveneWordId;
use sqlx::PgConnection;

use super::SloveneWordModel;
use crate::{
    entities::{
        self,
        EnglishWordModel,
        InternalSloveneWordReducedModel,
        InternalWordModel,
        WordLanguage,
    },
    QueryError,
    QueryResult,
};


#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NewSloveneWord {
    pub lemma: String,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SloveneWordFieldsToUpdate {
    pub new_lemma: Option<String>,
}




pub struct SloveneWordMutation;

impl SloveneWordMutation {
    pub async fn create(
        database_connection: &mut PgConnection,
        word_to_create: NewSloveneWord,
    ) -> QueryResult<SloveneWordModel> {
        let new_word_id = SloveneWordId::generate();
        let new_word_language_code = WordLanguage::Slovene.to_ietf_bcp_47_language_tag();
        let new_word_created_at = Utc::now();
        let new_word_last_modified_at = new_word_created_at;

        let bare_word_model = sqlx::query_as!(
            InternalWordModel,
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
            InternalSloveneWordReducedModel,
            "INSERT INTO kolomoni.word_slovene (word_id, lemma) \
                VALUES ($1, $2) \
                RETURNING word_id, lemma",
            new_word_id.into_uuid(),
            &word_to_create.lemma,
        )
        .fetch_one(database_connection)
        .await?;


        Ok(SloveneWordModel {
            word_id: SloveneWordId::new(english_word_model.word_id),
            lemma: english_word_model.lemma,
            created_at: bare_word_model.created_at,
            last_modified_at: bare_word_model.last_modified_at,
        })
    }

    pub async fn update(
        database_connection: &mut PgConnection,
        slovene_word_id: SloveneWordId,
        fields_to_update: SloveneWordFieldsToUpdate,
    ) -> QueryResult<bool> {
        let Some(new_lemma) = fields_to_update.new_lemma else {
            return Ok(true);
        };


        let slovene_word_id = slovene_word_id.into_uuid();

        let query_result = sqlx::query!(
            "UPDATE kolomoni.word_slovene \
                SET lemma = $1 \
                WHERE word_id = $2",
            new_lemma,
            slovene_word_id
        )
        .execute(database_connection)
        .await?;


        if query_result.rows_affected() > 1 {
            return Err(QueryError::database_inconsistency(
                "more than one row was affected when updating a slovene word",
            ));
        }

        Ok(query_result.rows_affected() == 1)
    }

    pub async fn delete(
        database_connection: &mut PgConnection,
        slovene_word_id: SloveneWordId,
    ) -> QueryResult<bool> {
        // TODO refactor this and EnglishWordMutation::delete to forward to EnglishWord::delete
        let word_uuid = slovene_word_id.into_uuid();

        let query_result = sqlx::query!(
            "DELETE FROM kolomoni.word \
                WHERE id = $1",
            word_uuid
        )
        .execute(database_connection)
        .await?;

        if query_result.rows_affected() > 1 {
            return Err(QueryError::database_inconsistency(
                "more than one row was affected when deleting an english word",
            ));
        }

        Ok(query_result.rows_affected() == 1)
    }
}
