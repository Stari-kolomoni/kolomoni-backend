use kolomoni_core::api_models::{
    EnglishWord,
    EnglishWordMeaning,
    EnglishWordMeaningWithDetails,
    EnglishWordMeaningWithShallowDetails,
    EnglishWordWithMeanings,
    SloveneTranslation,
};
use kolomoni_database::entities;

use crate::api::traits::IntoApiModel;


/*
 * Impls for the "word" part of the endpoints (word meanings are below).
 */

impl IntoApiModel<EnglishWord> for entities::word_english::EnglishWordModel {
    fn into_api_model(self) -> EnglishWord {
        let english_word_id = self.id();
        let (word, english_word) = self.into_inner();

        EnglishWord {
            id: english_word_id,
            created_at: word.created_at,
            last_modified_at: word.last_modified_at,
            lemma: english_word.lemma,
        }
    }
}

impl IntoApiModel<EnglishWordWithMeanings> for entities::word_english::EnglishWordWithMeaningsModel {
    fn into_api_model(self) -> EnglishWordWithMeanings {
        let english_word_id = self.id();
        let (word, english_word, meanings) = self.into_inner();

        let meanings = meanings
            .into_iter()
            .map(IntoApiModel::into_api_model)
            .collect();

        EnglishWordWithMeanings {
            id: english_word_id,
            created_at: word.created_at,
            last_modified_at: word.last_modified_at,
            lemma: english_word.lemma,
            meanings,
        }
    }
}

/*
 * Impls for the "word meaning" part of the endpoints (words themselves are above).
 */


impl IntoApiModel<EnglishWordMeaning> for entities::word_meaning_english::EnglishWordMeaningModel {
    fn into_api_model(self) -> EnglishWordMeaning {
        let english_word_meaning_id = self.id();
        let (word_meaning, english_word_meaning) = self.into_inner();

        EnglishWordMeaning {
            word_meaning_id: english_word_meaning_id,
            created_at: word_meaning.created_at,
            last_modified_at: word_meaning.last_modified_at,
            abbreviation: english_word_meaning.abbreviation,
            description: english_word_meaning.description,
            disambiguation: english_word_meaning.disambiguation,
        }
    }
}


impl IntoApiModel<EnglishWordMeaningWithShallowDetails>
    for entities::word_meaning_english::EnglishWordMeaningModelWithShallowDetails
{
    fn into_api_model(self) -> EnglishWordMeaningWithShallowDetails {
        let english_word_meaning_id = self.id();
        let (word_meaning, english_word_meaning, categories) = self.into_inner();

        EnglishWordMeaningWithShallowDetails {
            word_meaning_id: english_word_meaning_id,
            created_at: word_meaning.created_at,
            last_modified_at: word_meaning.last_modified_at,
            abbreviation: english_word_meaning.abbreviation,
            disambiguation: english_word_meaning.disambiguation,
            description: english_word_meaning.description,
            categories,
        }
    }
}

impl IntoApiModel<EnglishWordMeaningWithDetails>
    for entities::word_meaning_english::EnglishWordMeaningModelWithDetails
{
    fn into_api_model(self) -> EnglishWordMeaningWithDetails {
        let english_word_meaning_id = self.id();
        let (word_meaning, english_word_meaning, categories, translations) = self.into_inner();

        let translations = translations
            .into_iter()
            .map(IntoApiModel::into_api_model)
            .collect();

        EnglishWordMeaningWithDetails {
            word_meaning_id: english_word_meaning_id,
            created_at: word_meaning.created_at,
            last_modified_at: word_meaning.last_modified_at,
            abbreviation: english_word_meaning.abbreviation,
            disambiguation: english_word_meaning.disambiguation,
            description: english_word_meaning.description,
            categories,
            translations,
        }
    }
}


impl IntoApiModel<SloveneTranslation> for entities::word_meaning_english::SloveneTranslationModel {
    fn into_api_model(self) -> SloveneTranslation {
        SloveneTranslation {
            word: self.word.into_api_model(),
            word_meaning: self.word_meaning.into_api_model(),
            translated_at: self.translated_at,
            translated_by: self.translated_by,
        }
    }
}
