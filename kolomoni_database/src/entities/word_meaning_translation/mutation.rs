use chrono::Utc;
use kolomoni_core::id::{EnglishWordMeaningId, SloveneWordMeaningId, UserId};
use sqlx::PgConnection;

use super::WordMeaningTranslationModel;
use crate::{IntoExternalModel, QueryError, QueryResult};

pub struct WordMeaningTranslationMutation;

impl WordMeaningTranslationMutation {
    pub async fn create(
        database_connection: &mut PgConnection,
        english_word_meaning_id: EnglishWordMeaningId,
        slovene_word_meaning_id: SloveneWordMeaningId,
        translated_by: Option<UserId>,
    ) -> QueryResult<WordMeaningTranslationModel> {
        let translated_at = Utc::now();

        let newly_created_translation = sqlx::query_as!(
            super::InternalWordMeaningTranslationModel,
            "INSERT INTO kolomoni.word_meaning_translation \
                (slovene_word_meaning_id, english_word_meaning_id, \
                 translated_at, translated_by) \
                VALUES ($1, $2, $3, $4) \
                RETURNING \
                    slovene_word_meaning_id, english_word_meaning_id, \
                    translated_at, translated_by",
            slovene_word_meaning_id.into_uuid(),
            english_word_meaning_id.into_uuid(),
            translated_at,
            translated_by.map(|id| id.into_uuid())
        )
        .fetch_one(database_connection)
        .await?;

        Ok(newly_created_translation.into_external_model())
    }

    pub async fn delete(
        database_connection: &mut PgConnection,
        english_word_meaning_id: EnglishWordMeaningId,
        slovene_word_meaning_id: SloveneWordMeaningId,
    ) -> QueryResult<bool> {
        let query_result = sqlx::query_scalar!(
            "DELETE FROM kolomoni.word_meaning_translation \
                WHERE slovene_word_meaning_id = $1 \
                    AND english_word_meaning_id = $2",
            slovene_word_meaning_id.into_uuid(),
            english_word_meaning_id.into_uuid(),
        )
        .execute(database_connection)
        .await?;

        if query_result.rows_affected() > 1 {
            return Err(QueryError::database_inconsistency(
                "more than one row was affected while deleting a translation",
            ));
        }


        Ok(query_result.rows_affected() == 1)
    }
}
