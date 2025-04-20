use kolomoni_core::ids::{EnglishWordMeaningId, SloveneWordMeaningId};
use kolomoni_database::entities::{
    word_english::EnglishWordModel,
    word_meaning_english::EnglishWordMeaningModel,
    word_meaning_slovene::SloveneWordMeaningModel,
    word_slovene::SloveneWordModel,
};

/// A change event in relation to english words, slovene words and categories
/// (see [`kolomoni_search`] crate).
///
/// The variants of this enum are used as a message that is sent to the [`WordIndexChangeHandler`]
/// in order to signal that something has changed in the database and needs to be reindexed/recached.
#[derive(Clone)]
pub enum SearchIndexModificationMessage {
    // A hint to clean and reindex the entire collection.
    PerformFullReindex,

    EnglishWordMeaningCreatedOrUpdated {
        english_word_meaning: EnglishWordMeaningModel,
        english_word: EnglishWordModel,
    },

    EnglishWordMeaningRemoved {
        english_word_meaning_id: EnglishWordMeaningId,
    },

    SloveneWordMeaningCreatedOrUpdated {
        slovene_word_meaning: SloveneWordMeaningModel,
        slovene_word: SloveneWordModel,
    },

    SloveneWordMeaningRemoved {
        slovene_word_meaning_id: SloveneWordMeaningId,
    },
}
