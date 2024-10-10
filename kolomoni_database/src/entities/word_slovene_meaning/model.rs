use std::borrow::Cow;

use chrono::{DateTime, Utc};
use kolomoni_core::ids::{CategoryId, EnglishWordMeaningId, SloveneWordMeaningId, UserId};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    entities::InternalCategoryIdOnlyModel,
    IntoExternalModel,
    TryIntoStronglyTypedInternalModel,
};



pub struct SloveneWordMeaningModel {
    pub id: SloveneWordMeaningId,

    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,
}


pub struct SloveneWordMeaningModelWithCategoriesAndTranslations {
    pub id: SloveneWordMeaningId,

    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,

    pub categories: Vec<CategoryId>,

    pub translates_into: Vec<TranslatesIntoEnglishWordMeaningModel>,
}


#[derive(Deserialize)]
pub struct InternalSloveneWordMeaningModelWithCategoriesAndTranslations {
    pub id: Uuid,

    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,

    pub categories: Vec<InternalCategoryIdOnlyModel>,

    pub translates_into: Vec<InternalTranslatesIntoEnglishWordModel>,
}

impl IntoExternalModel for InternalSloveneWordMeaningModelWithCategoriesAndTranslations {
    type ExternalModel = SloveneWordMeaningModelWithCategoriesAndTranslations;

    fn into_external_model(self) -> Self::ExternalModel {
        Self::ExternalModel {
            id: SloveneWordMeaningId::new(self.id),
            disambiguation: self.disambiguation,
            abbreviation: self.abbreviation,
            description: self.description,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
            categories: self
                .categories
                .into_iter()
                .map(|internal_category| internal_category.into_external_model())
                .collect(),
            translates_into: self
                .translates_into
                .into_iter()
                .map(|internal_translation| internal_translation.into_external_model())
                .collect(),
        }
    }
}


pub struct SloveneWordMeaningModelWithWeaklyTypedCategoriesAndTranslations {
    pub word_meaning_id: Uuid,

    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,

    pub categories: serde_json::Value,

    pub translates_into: serde_json::Value,
}

impl TryIntoStronglyTypedInternalModel
    for SloveneWordMeaningModelWithWeaklyTypedCategoriesAndTranslations
{
    type InternalModel = InternalSloveneWordMeaningModelWithCategoriesAndTranslations;
    type Error = Cow<'static, str>;

    fn try_into_strongly_typed_internal_model(self) -> Result<Self::InternalModel, Self::Error> {
        let internal_categories = serde_json::from_value::<Vec<InternalCategoryIdOnlyModel>>(
            self.categories,
        )
        .map_err(|error| {
            Cow::from(format!(
                "failed to parse returned JSON as internal ID-only categories model: {}",
                error
            ))
        })?;

        let internal_translates_into = serde_json::from_value::<
            Vec<InternalTranslatesIntoEnglishWordModel>,
        >(self.translates_into)
        .map_err(|error| {
            Cow::from(format!(
                "failed to parse returned JSON as internal english translations model: {}",
                error
            ))
        })?;


        Ok(Self::InternalModel {
            id: self.word_meaning_id,
            disambiguation: self.disambiguation,
            abbreviation: self.abbreviation,
            description: self.description,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
            categories: internal_categories,
            translates_into: internal_translates_into,
        })
    }
}



pub struct TranslatesIntoEnglishWordMeaningModel {
    pub word_meaning_id: EnglishWordMeaningId,

    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,

    pub categories: Vec<CategoryId>,

    pub translated_at: DateTime<Utc>,

    pub translated_by: Option<UserId>,
}

#[derive(Deserialize)]
pub struct InternalTranslatesIntoEnglishWordModel {
    pub word_meaning_id: Uuid,

    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,

    pub categories: Vec<InternalCategoryIdOnlyModel>,

    pub translated_at: DateTime<Utc>,

    pub translated_by: Option<Uuid>,
}

impl IntoExternalModel for InternalTranslatesIntoEnglishWordModel {
    type ExternalModel = TranslatesIntoEnglishWordMeaningModel;

    fn into_external_model(self) -> Self::ExternalModel {
        Self::ExternalModel {
            word_meaning_id: EnglishWordMeaningId::new(self.word_meaning_id),
            disambiguation: self.disambiguation,
            abbreviation: self.abbreviation,
            description: self.description,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
            categories: self
                .categories
                .into_iter()
                .map(|internal_model| internal_model.into_external_model())
                .collect(),
            translated_at: self.translated_at,
            translated_by: self.translated_by.map(UserId::new),
        }
    }
}




pub struct InternalSloveneWordMeaningModel {
    pub(crate) word_meaning_id: Uuid,

    pub(crate) disambiguation: Option<String>,

    pub(crate) abbreviation: Option<String>,

    pub(crate) description: Option<String>,

    pub(crate) created_at: DateTime<Utc>,

    pub(crate) last_modified_at: DateTime<Utc>,
}

impl IntoExternalModel for InternalSloveneWordMeaningModel {
    type ExternalModel = SloveneWordMeaningModel;

    fn into_external_model(self) -> Self::ExternalModel {
        let id = SloveneWordMeaningId::new(self.word_meaning_id);

        Self::ExternalModel {
            id,
            disambiguation: self.disambiguation,
            abbreviation: self.abbreviation,
            description: self.description,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
        }
    }
}
