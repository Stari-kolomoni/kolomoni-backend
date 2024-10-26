use kolomoni_core::ids::{CategoryId, WordId, WordMeaningId};
use sqlx::PgConnection;

use crate::QueryResult;

pub struct WordMeaningCategoryQuery;

impl WordMeaningCategoryQuery {
    pub async fn exists_by_word_meaning_and_category_id(
        database_connection: &mut PgConnection,
        word_id: WordId,
        word_meaning_id: WordMeaningId,
        category_id: CategoryId,
    ) -> QueryResult<bool> {
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS (\
                SELECT 1 \
                    FROM kolomoni.word_meaning_category wmc \
                    INNER JOIN kolomoni.word_meaning wm \
                        ON wmc.word_meaning_id = wm.id \
                    WHERE wm.word_id = $1 \
                        AND wmc.word_meaning_id = $2 \
                        AND wmc.category_id = $3
            )",
            word_id.into_uuid(),
            word_meaning_id.into_uuid(),
            category_id.into_uuid(),
        )
        .fetch_one(database_connection)
        .await?;

        Ok(exists.unwrap_or(false))
    }
}
