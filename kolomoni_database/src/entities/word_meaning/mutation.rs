use chrono::{DateTime, Utc};
use kolomoni_core::ids::{WordId, WordMeaningId};
use sqlx::PgConnection;

use super::WordMeaningModel;
use crate::{
    entities::word_meaning::internal::InternalWordMeaningModel,
    IntoExternalModel,
    QueryResult,
};

pub struct WordMeaningMutation;


pub struct NewWordMeaning {
    pub word_id: WordId,
    pub created_at: DateTime<Utc>,
    pub last_modified_at: DateTime<Utc>,
}


impl WordMeaningMutation {
    pub async fn create(
        database_connection: &mut PgConnection,
        word_meaning: NewWordMeaning,
    ) -> QueryResult<WordMeaningModel> {
        let new_word_meaning_id = WordMeaningId::generate();

        let new_internal_word_meaning = sqlx::query_as!(
            InternalWordMeaningModel,
            "INSERT INTO kolomoni.word_meaning \
                    (id, word_id, created_at, last_modified_at) \
                VALUES \
                    ($1, $2, $3, $4) \
                RETURNING \
                    id, word_id, created_at, last_modified_at",
            new_word_meaning_id.into_uuid(),
            word_meaning.word_id.into_uuid(),
            word_meaning.created_at,
            word_meaning.last_modified_at
        )
        .fetch_one(database_connection)
        .await?;

        Ok(new_internal_word_meaning.into_external_model())
    }
}
