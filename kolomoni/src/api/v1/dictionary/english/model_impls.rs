use kolomoni_core::api_models::{
    EnglishWordMeaning,
    EnglishWordMeaningWithCategoriesAndTranslations,
    EnglishWordWithMeanings,
    ShallowSloveneWordMeaning,
};
use kolomoni_database::entities;

use crate::api::traits::IntoApiModel;


/*
 * Impls for the "word" part of the endpoints (word meanings are below).
 */



impl IntoApiModel<EnglishWordMeaningWithCategoriesAndTranslations>
    for entities::EnglishWordMeaningModelWithCategoriesAndTranslations
{
    fn into_api_model(self) -> EnglishWordMeaningWithCategoriesAndTranslations {
        EnglishWordMeaningWithCategoriesAndTranslations {
            meaning_id: self.id,
            disambiguation: self.disambiguation,
            abbreviation: self.abbreviation,
            description: self.description,
            categories: self.categories,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
            translates_into: self
                .translates_into
                .into_iter()
                .map(|internal_model| internal_model.into_api_model())
                .collect(),
        }
    }
}


impl IntoApiModel<ShallowSloveneWordMeaning> for entities::TranslatesIntoSloveneWordModel {
    fn into_api_model(self) -> ShallowSloveneWordMeaning {
        ShallowSloveneWordMeaning {
            meaning_id: self.word_meaning_id,
            disambiguation: self.disambiguation,
            abbreviation: self.abbreviation,
            description: self.description,
            categories: self.categories,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
        }
    }
}


impl IntoApiModel<EnglishWordWithMeanings> for entities::EnglishWordWithMeaningsModel {
    fn into_api_model(self) -> EnglishWordWithMeanings {
        let meanings = self
            .meanings
            .into_iter()
            .map(|meaning| meaning.into_api_model())
            .collect();

        EnglishWordWithMeanings {
            id: self.word_id,
            lemma: self.lemma,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
            meanings,
        }
    }
}

impl IntoApiModel<EnglishWordWithMeanings> for entities::EnglishWordModel {
    fn into_api_model(self) -> EnglishWordWithMeanings {
        EnglishWordWithMeanings {
            id: self.word_id,
            lemma: self.lemma,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
            meanings: vec![],
        }
    }
}


/*
 * Impls for the "word meaning" part of the endpoints (words themselves are above).
 */

impl IntoApiModel<EnglishWordMeaning> for entities::EnglishWordMeaningModel {
    fn into_api_model(self) -> EnglishWordMeaning {
        EnglishWordMeaning {
            meaning_id: self.id,
            disambiguation: self.disambiguation,
            abbreviation: self.abbreviation,
            description: self.description,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
        }
    }
}
