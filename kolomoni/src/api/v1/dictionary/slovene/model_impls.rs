use kolomoni_core::api_models::{
    ShallowEnglishWordMeaning,
    SloveneWordMeaning,
    SloveneWordMeaningWithCategoriesAndTranslations,
    SloveneWordWithMeanings,
};
use kolomoni_database::entities;

use crate::api::traits::IntoApiModel;




/*
 * Impls for the "word" part of the endpoints (word meanings are below).
 */


impl IntoApiModel<SloveneWordMeaningWithCategoriesAndTranslations>
    for entities::SloveneWordMeaningModelWithCategoriesAndTranslations
{
    fn into_api_model(self) -> SloveneWordMeaningWithCategoriesAndTranslations {
        SloveneWordMeaningWithCategoriesAndTranslations {
            meaning_id: self.id,
            disambiguation: self.disambiguation,
            abbreviation: self.abbreviation,
            description: self.description,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
            categories: self.categories,
            translates_into: self
                .translates_into
                .into_iter()
                .map(|translation| translation.into_api_model())
                .collect(),
        }
    }
}



impl IntoApiModel<ShallowEnglishWordMeaning> for entities::TranslatesIntoEnglishWordMeaningModel {
    fn into_api_model(self) -> ShallowEnglishWordMeaning {
        ShallowEnglishWordMeaning {
            meaning_id: self.word_meaning_id,
            disambiguation: self.disambiguation,
            abbreviation: self.abbreviation,
            description: self.description,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
            categories: self.categories,
        }
    }
}



impl IntoApiModel<SloveneWordWithMeanings> for entities::SloveneWordWithMeaningsModel {
    fn into_api_model(self) -> SloveneWordWithMeanings {
        SloveneWordWithMeanings {
            id: self.word_id,
            lemma: self.lemma,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
            meanings: self
                .meanings
                .into_iter()
                .map(|meaning| meaning.into_api_model())
                .collect(),
        }
    }
}



impl IntoApiModel<SloveneWordWithMeanings> for entities::SloveneWordModel {
    fn into_api_model(self) -> SloveneWordWithMeanings {
        SloveneWordWithMeanings {
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

impl IntoApiModel<SloveneWordMeaning> for entities::SloveneWordMeaningModel {
    fn into_api_model(self) -> SloveneWordMeaning {
        SloveneWordMeaning {
            meaning_id: self.id,
            disambiguation: self.disambiguation,
            abbreviation: self.abbreviation,
            description: self.description,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
        }
    }
}
