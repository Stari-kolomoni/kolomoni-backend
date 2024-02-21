use chrono::Utc;
use miette::{miette, Context, IntoDiagnostic, Result};
use sea_orm::{ActiveModelTrait, ActiveValue, ConnectionTrait, TransactionTrait, TryIntoModel};
use uuid::Uuid;

use crate::{
    begin_transaction,
    entities::{word, word_slovene},
    shared::{generate_random_word_uuid, WordLanguage},
};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NewSloveneWord {
    pub lemma: String,
    pub disambiguation: Option<String>,
    pub description: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct UpdatedSloveneWord {
    pub lemma: Option<String>,
    pub disambiguation: Option<String>,
    pub description: Option<String>,
}



pub struct SloveneWordMutation;

impl SloveneWordMutation {
    pub async fn create<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        slovene_word: NewSloveneWord,
    ) -> Result<word_slovene::Model> {
        let transaction = begin_transaction!(database)?;

        let random_uuid = generate_random_word_uuid();
        let added_at = Utc::now();


        let active_word = word::ActiveModel {
            id: ActiveValue::Set(random_uuid),
            language: ActiveValue::Set(WordLanguage::Slovene.to_ietf_language_tag().to_string()),
        };

        active_word
            .insert(&transaction)
            .await
            .into_diagnostic()
            .wrap_err("Failed while inserting base word.")?;


        let active_slovene_word = word_slovene::ActiveModel {
            word_id: ActiveValue::Set(random_uuid),
            lemma: ActiveValue::Set(slovene_word.lemma),
            disambiguation: ActiveValue::Set(slovene_word.disambiguation),
            description: ActiveValue::Set(slovene_word.description),
            added_at: ActiveValue::Set(added_at.fixed_offset()),
            last_edited_at: ActiveValue::Set(added_at.fixed_offset()),
        };

        let new_slovene_word = active_slovene_word
            .insert(&transaction)
            .await
            .into_diagnostic()
            .wrap_err("Failed while inserting slovene word.")?;


        transaction
            .commit()
            .await
            .into_diagnostic()
            .wrap_err("Failed to commit english word creation transaction.")?;

        Ok(new_slovene_word)
    }

    pub async fn update<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        word_uuid: Uuid,
        update: UpdatedSloveneWord,
    ) -> Result<word_slovene::Model> {
        let mut active_word_model = word_slovene::ActiveModel {
            word_id: ActiveValue::Unchanged(word_uuid),
            last_edited_at: ActiveValue::Set(Utc::now().fixed_offset()),
            ..Default::default()
        };

        if let Some(updated_lemma) = update.lemma {
            active_word_model.lemma = ActiveValue::Set(updated_lemma);
        };

        if let Some(updated_disambiguation) = update.disambiguation {
            active_word_model.disambiguation = ActiveValue::Set(Some(updated_disambiguation));
        }

        if let Some(updated_description) = update.description {
            active_word_model.description = ActiveValue::Set(Some(updated_description));
        }


        let updated_active_word = active_word_model
            .save(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed to update slovene word.")?;

        let updated_word = updated_active_word
            .try_into_model()
            .into_diagnostic()
            .wrap_err("Failed to convert active slovene model to normal model.")?;


        Ok(updated_word)
    }

    pub async fn delete<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        word_uuid: Uuid,
    ) -> Result<()> {
        let active_word_model = word_slovene::ActiveModel {
            word_id: ActiveValue::Unchanged(word_uuid),
            ..Default::default()
        };

        let deletion_result = active_word_model
            .delete(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while trying to delete slovene word.")?;

        debug_assert!(deletion_result.rows_affected <= 1);
        if deletion_result.rows_affected != 1 {
            return Err(miette!("no word with the given UUID"));
        }

        Ok(())
    }
}
