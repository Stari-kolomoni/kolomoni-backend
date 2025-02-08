use kolomoni_core::api_models::{
    EnglishTranslation,
    SloveneWord,
    SloveneWordMeaning,
    SloveneWordMeaningWithDetails,
    SloveneWordMeaningWithShallowDetails,
    SloveneWordWithMeanings,
};
use kolomoni_database::entities;

use crate::api::traits::IntoApiModel;




/*
 * Impls for the "word" part of the endpoints (word meanings are below).
 */


impl IntoApiModel<SloveneWordWithMeanings> for entities::word_slovene::SloveneWordWithMeaningsModel {
    fn into_api_model(self) -> SloveneWordWithMeanings {
        let slovene_word_id = self.id();
        let (word, slovene_word, meanings) = self.into_inner();

        let meanings = meanings
            .into_iter()
            .map(IntoApiModel::into_api_model)
            .collect();

        SloveneWordWithMeanings {
            id: slovene_word_id,
            created_at: word.created_at,
            last_modified_at: word.last_modified_at,
            lemma: slovene_word.lemma,
            meanings,
        }
    }
}



impl IntoApiModel<SloveneWord> for entities::word_slovene::SloveneWordModel {
    fn into_api_model(self) -> SloveneWord {
        let slovene_word_id = self.id();
        let (word, slovene_word) = self.into_inner();

        SloveneWord {
            id: slovene_word_id,
            created_at: word.created_at,
            last_modified_at: word.last_modified_at,
            lemma: slovene_word.lemma,
        }
    }
}



/*
 * Impls for the "word meaning" part of the endpoints (words themselves are above).
 */


impl IntoApiModel<SloveneWordMeaning> for entities::word_meaning_slovene::SloveneWordMeaningModel {
    fn into_api_model(self) -> SloveneWordMeaning {
        let slovene_word_meaning_id = self.word_meaning_id();
        let (word_meaning, slovene_word_meaning) = self.into_inner();

        SloveneWordMeaning {
            word_meaning_id: slovene_word_meaning_id,
            created_at: word_meaning.created_at,
            last_modified_at: word_meaning.last_modified_at,
            abbreviation: slovene_word_meaning.abbreviation,
            description: slovene_word_meaning.description,
            disambiguation: slovene_word_meaning.disambiguation,
        }
    }
}


impl IntoApiModel<SloveneWordMeaningWithShallowDetails>
    for entities::word_meaning_slovene::SloveneWordMeaningModelWithShallowDetails
{
    fn into_api_model(self) -> SloveneWordMeaningWithShallowDetails {
        let slovene_word_meaning_id = self.word_meaning_id();
        let (word_meaning, slovene_word_meaning, categories) = self.into_inner();

        SloveneWordMeaningWithShallowDetails {
            word_meaning_id: slovene_word_meaning_id,
            created_at: word_meaning.created_at,
            last_modified_at: word_meaning.last_modified_at,
            disambiguation: slovene_word_meaning.disambiguation,
            abbreviation: slovene_word_meaning.abbreviation,
            description: slovene_word_meaning.description,
            categories,
        }
    }
}


impl IntoApiModel<SloveneWordMeaningWithDetails>
    for entities::word_meaning_slovene::SloveneWordMeaningModelWithDetails
{
    fn into_api_model(self) -> SloveneWordMeaningWithDetails {
        let slovene_word_meaning_id = self.word_meaning_id();
        let (word_meaning, slovene_word_meaning, categories, translations) = self.into_inner();

        let translations = translations
            .into_iter()
            .map(IntoApiModel::into_api_model)
            .collect();

        SloveneWordMeaningWithDetails {
            word_meaning_id: slovene_word_meaning_id,
            created_at: word_meaning.created_at,
            last_modified_at: word_meaning.last_modified_at,
            disambiguation: slovene_word_meaning.disambiguation,
            abbreviation: slovene_word_meaning.abbreviation,
            description: slovene_word_meaning.description,
            categories,
            translations,
        }
    }
}

impl IntoApiModel<EnglishTranslation> for entities::word_meaning_slovene::EnglishTranslationModel {
    fn into_api_model(self) -> EnglishTranslation {
        EnglishTranslation {
            word: self.word.into_api_model(),
            word_meaning: self.word_meaning.into_api_model(),
            translated_at: self.translated_at,
            translated_by: self.translated_by,
        }
    }
}
