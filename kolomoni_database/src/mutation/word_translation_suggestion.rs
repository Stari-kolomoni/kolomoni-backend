use chrono::Utc;
use miette::{Context, IntoDiagnostic, Result};
use sea_orm::{ActiveModelTrait, ActiveValue, ConnectionTrait, TransactionTrait};
use uuid::Uuid;

use crate::entities::word_translation_suggestion;



pub struct NewTranslationSuggestion {
    pub english_word_id: Uuid,
    pub slovene_word_id: Uuid,
}

pub struct TranslationSuggestionToDelete {
    pub english_word_id: Uuid,
    pub slovene_word_id: Uuid,
}


pub struct TranslationSuggestionMutation;

impl TranslationSuggestionMutation {
    pub async fn create<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        new_translation_suggestion: NewTranslationSuggestion,
    ) -> Result<word_translation_suggestion::Model> {
        let active_suggestion = word_translation_suggestion::ActiveModel {
            english_word_id: ActiveValue::Set(new_translation_suggestion.english_word_id),
            slovene_word_id: ActiveValue::Set(new_translation_suggestion.slovene_word_id),
            suggested_at: ActiveValue::Set(Utc::now().fixed_offset()),
        };

        let new_suggestion_model = active_suggestion
            .insert(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while inserting new translation suggestion into the database.")?;

        Ok(new_suggestion_model)
    }

    pub async fn delete<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        to_delete: TranslationSuggestionToDelete,
    ) -> Result<()> {
        let active_suggestion = word_translation_suggestion::ActiveModel {
            english_word_id: ActiveValue::Unchanged(to_delete.english_word_id),
            slovene_word_id: ActiveValue::Unchanged(to_delete.slovene_word_id),
            ..Default::default()
        };


        active_suggestion
            .delete(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while deleting translation suggestion from the database.")?;

        Ok(())
    }
}
