use chrono::Utc;
use kolomoni_core::ids::SloveneWordId;
use sqlx::PgConnection;

use super::{internal::InternalSloveneWordModel, SloveneWordModel};
use crate::{
    entities::{
        word::{NewWord, WordLanguage, WordMutation},
        word_slovene::internal_insert_only::InsertOnlyInternalSloveneWordModel,
    },
    IntoExternalModel,
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
        let new_word_created_at = Utc::now();
        let new_word_last_modified_at = new_word_created_at;


        let new_word = WordMutation::create(
            database_connection,
            NewWord {
                language: WordLanguage::Slovene,
                created_at: new_word_created_at,
                last_modified_at: new_word_last_modified_at,
            },
        )
        .await?;

        let english_word_model = sqlx::query_as!(
            InsertOnlyInternalSloveneWordModel,
            "INSERT INTO kolomoni.word_slovene \
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


        let complete_internal_model = InternalSloveneWordModel {
            word_id: new_word.id.into_uuid(),
            created_at: new_word.created_at,
            last_modified_at: new_word.last_modified_at,
            lemma: english_word_model.lemma,
        };


        Ok(complete_internal_model.into_external_model())
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
        WordMutation::delete(database_connection, slovene_word_id.upcast_to_word_id()).await
    }
}
