use kolomoni_core::ids::WordId;
use sqlx::PgConnection;

use crate::{QueryError, QueryResult};


pub struct WordMutation;

impl WordMutation {
    pub async fn delete(
        database_connection: &mut PgConnection,
        word_id: WordId,
    ) -> QueryResult<bool> {
        let word_uuid = word_id.into_uuid();

        let query_result = sqlx::query!(
            "DELETE FROM kolomoni.word \
                WHERE id = $1",
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
