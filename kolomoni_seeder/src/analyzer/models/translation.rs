use thiserror::Error;

use super::{
    english_word_meaning::EnglishWordMeaning,
    slovene_word_meaning::SloveneWordMeaning,
    InternalEnglishWordMeaningId,
    InternalSloveneWordMeaningId,
    InternalTranslationId,
};
use crate::analyzer::insert_only_set::{GrowingKeyedSet, InternalId};


#[derive(Debug, Error)]
#[error("unable to find english word meaning by internal ID: {}", .internal_english_word_meaning_id)]
pub struct EnglishWordMeaningNotFound {
    internal_english_word_meaning_id: InternalEnglishWordMeaningId,
}

#[derive(Debug, Error)]
#[error("unable to find slovene word meaning by internal ID: {}", .internal_slovene_word_meaning_id)]
pub struct SloveneWordMeaningNotFound {
    internal_slovene_word_meaning_id: InternalSloveneWordMeaningId,
}


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

    #[allow(dead_code)]
    pub fn english_word_meaning<'s>(
        &self,
        growing_english_word_meaning_set: &'s GrowingKeyedSet<EnglishWordMeaning>,
    ) -> Result<&'s EnglishWordMeaning, EnglishWordMeaningNotFound> {
        growing_english_word_meaning_set
            .get(&self.english_word_meaning)
            .ok_or(EnglishWordMeaningNotFound {
                internal_english_word_meaning_id: self.english_word_meaning,
            })
    }

    #[allow(dead_code)]
    pub fn slovene_word_meaning<'s>(
        &self,
        growing_slovene_word_meaning_set: &'s GrowingKeyedSet<SloveneWordMeaning>,
    ) -> Result<&'s SloveneWordMeaning, SloveneWordMeaningNotFound> {
        growing_slovene_word_meaning_set
            .get(&self.slovene_word_meaning)
            .ok_or(SloveneWordMeaningNotFound {
                internal_slovene_word_meaning_id: self.slovene_word_meaning,
            })
    }
}


impl InternalId for Translation {
    type InternalId = InternalTranslationId;

    fn internal_id(&self) -> Self::InternalId {
        self.internal_id
    }
}
