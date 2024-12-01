use parking_lot::MappedRwLockReadGuard;
use thiserror::Error;

use super::{
    category::Category,
    english_word::EnglishWord,
    english_word_meaning::{
        EnglishWordMeaning,
        EnglishWordMeaningResolutionContext,
        EnglishWordMeaningResolutionError,
        ResolvedEnglishWordMeaning,
    },
    slovene_word::SloveneWord,
    slovene_word_meaning::{
        ResolvedSloveneWordMeaning,
        SloveneWordMeaning,
        SloveneWordMeaningResolutionContext,
        SloveneWordMeaningResolutionError,
    },
    InternalEnglishWordMeaningId,
    InternalSloveneWordMeaningId,
    InternalTranslationId,
    TryToResolvedModelWithContext,
};
use crate::analyzer::insert_only_set::{GrowingKeyedSet, InternalId};


pub struct Translation {
    internal_id: InternalTranslationId,

    pub english_word_meaning: InternalEnglishWordMeaningId,
    pub slovene_word_meaning: InternalSloveneWordMeaningId,
}

impl Translation {
    pub fn new(
        internal_english_word_meaning_id: InternalEnglishWordMeaningId,
        internal_slovene_word_meaning_id: InternalSloveneWordMeaningId,
    ) -> Self {
        Self {
            internal_id: InternalTranslationId::generate(),
            english_word_meaning: internal_english_word_meaning_id,
            slovene_word_meaning: internal_slovene_word_meaning_id,
        }
    }
}


impl InternalId for Translation {
    type InternalId = InternalTranslationId;

    fn internal_id(&self) -> Self::InternalId {
        self.internal_id
    }
}


pub struct ResolvedTranslation {
    internal_id: InternalTranslationId,

    pub english_word_meaning: ResolvedEnglishWordMeaning,
    pub slovene_word_meaning: ResolvedSloveneWordMeaning,
}

impl InternalId for ResolvedTranslation {
    type InternalId = InternalTranslationId;

    fn internal_id(&self) -> Self::InternalId {
        self.internal_id
    }
}



pub struct TranslationResolutionContext<'a> {
    english_words: &'a GrowingKeyedSet<EnglishWord>,
    english_word_meanings: &'a GrowingKeyedSet<EnglishWordMeaning>,
    slovene_words: &'a GrowingKeyedSet<SloveneWord>,
    slovene_word_meanings: &'a GrowingKeyedSet<SloveneWordMeaning>,
    categories: &'a GrowingKeyedSet<Category>,
}

impl<'a> TranslationResolutionContext<'a> {
    fn english_word_meaning_by_internal_id(
        &self,
        internal_english_word_meaning_id: &InternalEnglishWordMeaningId,
    ) -> Option<MappedRwLockReadGuard<'a, EnglishWordMeaning>> {
        self.english_word_meanings
            .get(internal_english_word_meaning_id)
    }

    fn slovene_word_meaning_by_internal_id(
        &self,
        internal_slovene_word_meaning_id: &InternalSloveneWordMeaningId,
    ) -> Option<MappedRwLockReadGuard<'a, SloveneWordMeaning>> {
        self.slovene_word_meanings
            .get(internal_slovene_word_meaning_id)
    }
}


#[derive(Debug, Error)]
pub enum TranslationResolutionError {
    #[error("no such english word meaning with internal ID: {}", .internal_english_word_meaning_id)]
    EnglishWordMeaningNotFound {
        internal_english_word_meaning_id: InternalEnglishWordMeaningId,
    },

    #[error("failed to resolve english word meaning")]
    EnglishWordMeaningResolutionError {
        #[from]
        #[source]
        error: EnglishWordMeaningResolutionError,
    },

    #[error("no such slovene word meaning with internal ID: {}", .internal_slovene_word_meaning_id)]
    SloveneWordMeaningNotFound {
        internal_slovene_word_meaning_id: InternalSloveneWordMeaningId,
    },

    #[error("failed to resolve slovene word meaning")]
    SloveneWordMeaningResolutionError {
        #[from]
        #[source]
        error: SloveneWordMeaningResolutionError,
    },
}


impl TryToResolvedModelWithContext for Translation {
    type Context<'ctx> = TranslationResolutionContext<'ctx>;
    type ResolvedModel = ResolvedTranslation;
    type Error = TranslationResolutionError;

    fn try_to_resolved_model<'ctx>(
        &'ctx self,
        context: &'ctx Self::Context<'ctx>,
    ) -> Result<Self::ResolvedModel, Self::Error> {
        let Some(english_word_meaning) =
            context.english_word_meaning_by_internal_id(&self.english_word_meaning)
        else {
            return Err(
                TranslationResolutionError::EnglishWordMeaningNotFound {
                    internal_english_word_meaning_id: self.english_word_meaning,
                },
            );
        };

        let resolved_english_word_meaning = english_word_meaning.try_to_resolved_model(
            &EnglishWordMeaningResolutionContext::new(&context.english_words, &context.categories),
        )?;



        let Some(slovene_word_meaning) =
            context.slovene_word_meaning_by_internal_id(&self.slovene_word_meaning)
        else {
            return Err(
                TranslationResolutionError::SloveneWordMeaningNotFound {
                    internal_slovene_word_meaning_id: self.slovene_word_meaning,
                },
            );
        };

        let resolved_slovene_word_meaning = slovene_word_meaning.try_to_resolved_model(
            &SloveneWordMeaningResolutionContext::new(&context.slovene_words, &context.categories),
        )?;


        Ok(Self::ResolvedModel {
            internal_id: self.internal_id,
            english_word_meaning: resolved_english_word_meaning,
            slovene_word_meaning: resolved_slovene_word_meaning,
        })
    }
}
