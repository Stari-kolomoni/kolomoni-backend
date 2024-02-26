use chrono::{DateTime, Utc};
use miette::{Context, IntoDiagnostic, Result};
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

    pub async fn set_last_modified_at<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        word_uuid: Uuid,
        new_last_edited_at: DateTime<Utc>,
    ) -> Result<word_slovene::Model> {
        let active_word_model = word_slovene::ActiveModel {
            word_id: ActiveValue::Unchanged(word_uuid),
            last_edited_at: ActiveValue::Set(new_last_edited_at.fixed_offset()),
            ..Default::default()
        };

        let updated_word = active_word_model
            .update(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while setting last modified datetime for slovene word.")?;


        Ok(updated_word)
    }

    // For deletion, see [`WordMutation::delete`][super::word::WordMutation::delete].
}
