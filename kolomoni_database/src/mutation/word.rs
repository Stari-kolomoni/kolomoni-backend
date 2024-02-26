use miette::{miette, Context, IntoDiagnostic, Result};
use sea_orm::{ActiveModelTrait, ActiveValue, ConnectionTrait, TransactionTrait};
use uuid::Uuid;

use crate::entities;

pub struct WordMutation;

impl WordMutation {
    pub async fn delete<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        word_uuid: Uuid,
    ) -> Result<()> {
        let active_word_model = entities::word::ActiveModel {
            id: ActiveValue::Unchanged(word_uuid),
            ..Default::default()
        };

        let deletion_result = active_word_model
            .delete(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while trying to delete a word from the database.")?;

        debug_assert!(deletion_result.rows_affected <= 1);
        if deletion_result.rows_affected != 1 {
            return Err(miette!("no word with the given UUID"));
        }

        Ok(())
    }
}
