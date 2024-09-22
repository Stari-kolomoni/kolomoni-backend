use kolomoni_core::id::{EnglishWordMeaningId, SloveneWordMeaningId};
use sqlx::PgConnection;

use crate::QueryResult;

pub struct WordMeaningTranslationQuery;

impl WordMeaningTranslationQuery {
    pub async fn exists(
        database_connection: &mut PgConnection,
        english_word_meaning_id: EnglishWordMeaningId,
        slovene_word_meaning_id: SloveneWordMeaningId,
    ) -> QueryResult<bool> {
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS (\
                SELECT 1 FROM kolomoni.word_meaning_translation \
                    WHERE english_word_meaning_id = $1
                        AND slovene_word_meaning_id = $2
            )",
            english_word_meaning_id.into_uuid(),
            slovene_word_meaning_id.into_uuid(),
        )
        .fetch_one(database_connection)
        .await?;

        Ok(exists.unwrap_or(false))
    }

    // TODO
}
