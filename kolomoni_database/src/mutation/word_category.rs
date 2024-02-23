use miette::Result;
use miette::{Context, IntoDiagnostic};
use sea_orm::{ActiveModelTrait, ActiveValue, ConnectionTrait, TransactionTrait};
use uuid::Uuid;

use crate::entities::word_category;

pub struct WordCategoryMutation;

impl WordCategoryMutation {
    pub async fn add_category_to_word<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        word_uuid: Uuid,
        category_id: i32,
    ) -> Result<()> {
        let word_category_active_model = word_category::ActiveModel {
            word_id: ActiveValue::Set(word_uuid),
            category_id: ActiveValue::Set(category_id),
        };

        word_category_active_model
            .insert(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while inserting word category relationship.")?;

        Ok(())
    }

    pub async fn remove_category_from_word<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        word_uuid: Uuid,
        category_id: i32,
    ) -> Result<()> {
        let word_category_active_model = word_category::ActiveModel {
            word_id: ActiveValue::Unchanged(word_uuid),
            category_id: ActiveValue::Unchanged(category_id),
        };

        word_category_active_model
            .delete(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while deleting word category relationship.")?;

        Ok(())
    }
}
