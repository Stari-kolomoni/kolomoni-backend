use chrono::Utc;
use miette::{Context, IntoDiagnostic, Result};
use sea_orm::{ActiveModelTrait, ActiveValue, ConnectionTrait, TransactionTrait};
use uuid::Uuid;

use super::{EnglishWordMutation, SloveneWordMutation};
use crate::{begin_transaction, commit_transaction, entities::word_translation};



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
        let transaction = begin_transaction!(database)?;


        let active_translation = word_translation::ActiveModel {
            english_word_id: ActiveValue::Set(new_translation.english_word_id),
            slovene_word_id: ActiveValue::Set(new_translation.slovene_word_id),
            translated_at: ActiveValue::Set(Utc::now().fixed_offset()),
        };

        let new_translation_model = active_translation
            .insert(&transaction)
            .await
            .into_diagnostic()
            .wrap_err("Failed while inserting new translation into the database.")?;



        // Now update the `last_modified_at` values for both words as well.

        let new_last_modified_at = Utc::now();

        EnglishWordMutation::set_last_modified_at(
            &transaction,
            new_translation.english_word_id,
            new_last_modified_at,
        )
        .await
        .wrap_err("Failed to set last modified for english word after creating a translation.")?;

        SloveneWordMutation::set_last_modified_at(
            &transaction,
            new_translation.slovene_word_id,
            new_last_modified_at,
        )
        .await
        .wrap_err("Failed to set last modified for slovene word after creating a translation.")?;


        commit_transaction!(transaction)?;
        Ok(new_translation_model)
    }

    pub async fn delete<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        to_delete: TranslationToDelete,
    ) -> Result<()> {
        let transaction = begin_transaction!(database)?;


        let active_translation = word_translation::ActiveModel {
            english_word_id: ActiveValue::Unchanged(to_delete.english_word_id),
            slovene_word_id: ActiveValue::Unchanged(to_delete.slovene_word_id),
            ..Default::default()
        };


        active_translation
            .delete(&transaction)
            .await
            .into_diagnostic()
            .wrap_err("Failed while deleting translation from the database.")?;


        // Now update the `last_modified_at` values for both words as well.

        let new_last_modified_at = Utc::now();

        EnglishWordMutation::set_last_modified_at(
            &transaction,
            to_delete.english_word_id,
            new_last_modified_at,
        )
        .await
        .wrap_err("Failed to set last modified for english word after deleting a translation.")?;

        SloveneWordMutation::set_last_modified_at(
            &transaction,
            to_delete.slovene_word_id,
            new_last_modified_at,
        )
        .await
        .wrap_err("Failed to set last modified for slovene word after deleting a translation.")?;


        commit_transaction!(transaction)?;
        Ok(())
    }
}
