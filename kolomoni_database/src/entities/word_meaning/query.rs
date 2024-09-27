use kolomoni_core::id::{WordId, WordMeaningId};
use sqlx::PgConnection;

use crate::QueryResult;

pub struct WordMeaningQuery;

impl WordMeaningQuery {
    pub async fn exists_by_meaning_and_word_id(
        database_connection: &mut PgConnection,
        word_id: WordId,
        word_meaning_id: WordMeaningId,
    ) -> QueryResult<bool> {
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS (\
                SELECT 1 \
                    FROM kolomoni.word_meaning \
                    WHERE id = $1 AND word_id = $2 \
            )",
            word_meaning_id.into_uuid(),
            word_id.into_uuid()
        )
        .fetch_one(database_connection)
        .await?;

        Ok(exists.unwrap_or(false))
    }
}
