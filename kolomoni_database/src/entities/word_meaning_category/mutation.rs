use kolomoni_core::ids::{CategoryId, WordMeaningId};
use sqlx::PgConnection;

use super::WordMeaningCategory;
use crate::{
    entities::word_meaning_category::internal::InternalWordMeaningCategory,
    IntoExternalModel,
    QueryError,
    QueryResult,
};


pub struct WordMeaningCategoryMutation;

impl WordMeaningCategoryMutation {
    pub async fn link_category_with_word_meaning(
        database_connection: &mut PgConnection,
        word_meaning_id: WordMeaningId,
        category_id: CategoryId,
    ) -> QueryResult<WordMeaningCategory> {
        let internal_word_meaning_category = sqlx::query_as!(
            InternalWordMeaningCategory,
            "INSERT INTO kolomoni.word_meaning_category (word_meaning_id, category_id) \
                VALUES \
                    ($1, $2) \
                RETURNING \
                    word_meaning_id, category_id",
            word_meaning_id.into_uuid(),
            category_id.into_uuid()
        )
        .fetch_one(database_connection)
        .await?;

        Ok(internal_word_meaning_category.into_external_model())
    }

    pub async fn unlink_category_from_word_meaning(
        database_connection: &mut PgConnection,
        word_meaning_id: WordMeaningId,
        category_id: CategoryId,
    ) -> QueryResult<bool> {
        let query_result = sqlx::query!(
            "DELETE FROM kolomoni.word_meaning_category \
                WHERE word_meaning_id = $1 AND category_id = $2",
            word_meaning_id.into_uuid(),
            category_id.into_uuid()
        )
        .execute(database_connection)
        .await?;


        if query_result.rows_affected() > 1 {
            return Err(QueryError::database_inconsistency(
                "more than one row was affected when \
                unlinking a category from a word meaning",
            ));
        }

        Ok(query_result.rows_affected() == 1)
    }
}
