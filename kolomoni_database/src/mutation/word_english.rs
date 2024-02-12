use chrono::Utc;
use miette::{Context, IntoDiagnostic, Result};
use sea_orm::{ActiveModelTrait, ActiveValue, ConnectionTrait, TransactionTrait};

use crate::{
    begin_transaction,
    entities::{word, word_english},
    shared::{generate_random_word_uuid, WordLanguage},
};


pub struct NewEnglishWord {
    pub lemma: String,
    pub disambiguation: Option<String>,
    pub description: Option<String>,
}

pub struct EnglishWordMutation;

impl EnglishWordMutation {
    pub async fn create<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        english_word: NewEnglishWord,
    ) -> Result<word_english::Model> {
        let transaction = begin_transaction!(database)?;

        let random_uuid = generate_random_word_uuid();
        let added_at = Utc::now();


        let active_word = word::ActiveModel {
            id: ActiveValue::Set(random_uuid),
            language: ActiveValue::Set(WordLanguage::English.to_ietf_language_tag().to_string()),
        };

        active_word
            .insert(&transaction)
            .await
            .into_diagnostic()
            .wrap_err("Failed while inserting base word.")?;


        let active_english_word = word_english::ActiveModel {
            word_id: ActiveValue::Set(random_uuid),
            lemma: ActiveValue::Set(english_word.lemma),
            disambiguation: ActiveValue::Set(english_word.disambiguation),
            description: ActiveValue::Set(english_word.description),
            added_at: ActiveValue::Set(added_at.fixed_offset()),
            last_edited_at: ActiveValue::Set(added_at.fixed_offset()),
        };

        let new_english_word = active_english_word
            .insert(&transaction)
            .await
            .into_diagnostic()
            .wrap_err("Failed while inserting english word.")?;


        Ok(new_english_word)
    }
}
