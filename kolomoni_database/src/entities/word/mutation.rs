use chrono::{DateTime, Utc};
use kolomoni_core::ids::WordId;
use sqlx::PgConnection;

use super::{WordLanguage, WordModel};
use crate::{
    entities::word::internal::InternalWordModel,
    QueryError,
    QueryResult,
    TryIntoExternalModel,
};

pub struct NewWord {
    pub language: WordLanguage,
    pub created_at: DateTime<Utc>,
    pub last_modified_at: DateTime<Utc>,
}


pub struct WordMutation;

impl WordMutation {
    pub async fn create(
        database_connection: &mut PgConnection,
        word_to_create: NewWord,
    ) -> QueryResult<WordModel> {
        let new_word_id = WordId::generate();
        let new_word_language_code = word_to_create.language.to_ietf_bcp_47_language_tag();

        let internal_word_model = sqlx::query_as!(
            InternalWordModel,
            "INSERT INTO kolomoni.word \
                    (id, language_code, created_at, last_modified_at) \
                VALUES \
                    ($1, $2, $3, $4) \
                RETURNING \
                    id, language_code, created_at, last_modified_at",
            new_word_id.into_uuid(),
            new_word_language_code,
            word_to_create.created_at,
            word_to_create.last_modified_at,
        )
        .fetch_one(&mut *database_connection)
        .await?;

        internal_word_model
            .try_into_external_model()
            .map_err(QueryError::model_error)
    }

    pub async fn delete(
        database_connection: &mut PgConnection,
        word_id: WordId,
    ) -> QueryResult<bool> {
        let word_uuid = word_id.into_uuid();

        let query_result = sqlx::query!(
            "DELETE FROM kolomoni.word w \
                WHERE w.id = $1",
            word_uuid
        )
        .execute(database_connection)
        .await?;

        if query_result.rows_affected() > 1 {
            return Err(QueryError::database_inconsistency(
                "more than one row was affected when deleting a word",
            ));
        }

        Ok(query_result.rows_affected() == 1)
    }
}
