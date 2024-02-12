use chrono::Utc;
use miette::{Context, IntoDiagnostic, Result};
use sea_orm::{ActiveModelTrait, ActiveValue, ConnectionTrait, TransactionTrait};

use crate::{
    begin_transaction,
    entities::{word, word_slovene},
    shared::{generate_random_word_uuid, WordLanguage},
};

pub struct NewSloveneWord {
    pub lemma: String,
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


        Ok(new_slovene_word)
    }
}
