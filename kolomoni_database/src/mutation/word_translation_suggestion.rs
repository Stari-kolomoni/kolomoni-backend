use chrono::Utc;
use miette::{Context, IntoDiagnostic, Result};
use sea_orm::{ActiveModelTrait, ActiveValue, ConnectionTrait, TransactionTrait};
use uuid::Uuid;

use super::{EnglishWordMutation, SloveneWordMutation};
use crate::{begin_transaction, commit_transaction, entities::word_translation_suggestion};



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
        let transaction = begin_transaction!(database)?;


        let active_suggestion = word_translation_suggestion::ActiveModel {
            english_word_id: ActiveValue::Set(new_translation_suggestion.english_word_id),
            slovene_word_id: ActiveValue::Set(new_translation_suggestion.slovene_word_id),
            suggested_at: ActiveValue::Set(Utc::now().fixed_offset()),
        };

        let new_suggestion_model = active_suggestion
            .insert(&transaction)
            .await
            .into_diagnostic()
            .wrap_err("Failed while inserting new translation suggestion into the database.")?;



        // Now update the `last_modified_at` values for both words as well.

        let new_last_modified_at = Utc::now();

        EnglishWordMutation::set_last_modified_at(
            &transaction,
            new_translation_suggestion.english_word_id,
            new_last_modified_at,
        )
        .await
        .wrap_err("Failed to set last modified for english word after creating a suggestion.")?;

        SloveneWordMutation::set_last_modified_at(
            &transaction,
            new_translation_suggestion.slovene_word_id,
            new_last_modified_at,
        )
        .await
        .wrap_err("Failed to set last modified for slovene word after creating a suggestion.")?;



        commit_transaction!(transaction)?;
        Ok(new_suggestion_model)
    }

    pub async fn delete<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        to_delete: TranslationSuggestionToDelete,
    ) -> Result<()> {
        let transaction = begin_transaction!(database)?;


        let active_suggestion = word_translation_suggestion::ActiveModel {
            english_word_id: ActiveValue::Unchanged(to_delete.english_word_id),
            slovene_word_id: ActiveValue::Unchanged(to_delete.slovene_word_id),
            ..Default::default()
        };


        active_suggestion
            .delete(&transaction)
            .await
            .into_diagnostic()
            .wrap_err("Failed while deleting translation suggestion from the database.")?;



        // Now update the `last_modified_at` values for both words as well.

        let new_last_modified_at = Utc::now();

        EnglishWordMutation::set_last_modified_at(
            &transaction,
            to_delete.english_word_id,
            new_last_modified_at,
        )
        .await
        .wrap_err("Failed to set last modified for english word after deleting a suggestion.")?;

        SloveneWordMutation::set_last_modified_at(
            &transaction,
            to_delete.slovene_word_id,
            new_last_modified_at,
        )
        .await
        .wrap_err("Failed to set last modified for slovene word after deleting a suggestion.")?;


        commit_transaction!(transaction)?;
        Ok(())
    }
}
