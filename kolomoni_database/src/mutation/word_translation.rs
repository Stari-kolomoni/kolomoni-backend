use chrono::Utc;
use miette::{Context, IntoDiagnostic, Result};
use sea_orm::{ActiveModelTrait, ActiveValue, ConnectionTrait, TransactionTrait};
use uuid::Uuid;

use crate::entities::word_translation;



pub struct NewTranslation {
    pub english_word_id: Uuid,
    pub slovene_word_id: Uuid,
}

pub struct TranslationToDelete {
    pub english_word_id: Uuid,
    pub slovene_word_id: Uuid,
}


pub struct TranslationMutation;

impl TranslationMutation {
    pub async fn create<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        new_translation: NewTranslation,
    ) -> Result<word_translation::Model> {
        let active_translation = word_translation::ActiveModel {
            english_word_id: ActiveValue::Set(new_translation.english_word_id),
            slovene_word_id: ActiveValue::Set(new_translation.slovene_word_id),
            translated_at: ActiveValue::Set(Utc::now().fixed_offset()),
        };

        let new_translation_model = active_translation
            .insert(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while inserting new translation into the database.")?;

        Ok(new_translation_model)
    }

    pub async fn delete<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        to_delete: TranslationToDelete,
    ) -> Result<()> {
        let active_translation = word_translation::ActiveModel {
            english_word_id: ActiveValue::Unchanged(to_delete.english_word_id),
            slovene_word_id: ActiveValue::Unchanged(to_delete.slovene_word_id),
            ..Default::default()
        };


        active_translation
            .delete(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while deleting translation from the database.")?;

        Ok(())
    }
}
