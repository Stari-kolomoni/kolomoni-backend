use std::borrow::Cow;

use chrono::{DateTime, Utc};
use kolomoni_core::id::{CategoryId, EnglishWordMeaningId, SloveneWordMeaningId, UserId};
use serde::Deserialize;
use uuid::Uuid;

use crate::{IntoExternalModel, TryIntoStronglyTypedInternalModel};


// TODO These names are a mess, refactor.


pub struct EnglishWordMeaningModel {
    pub id: EnglishWordMeaningId,

    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,
}



pub struct EnglishWordMeaningModelWithCategoriesAndTranslations {
    pub id: EnglishWordMeaningId,

    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,

    pub categories: Vec<CategoryId>,

    pub translates_into: Vec<TranslatesIntoSloveneWordModel>,
}



pub struct TranslatesIntoSloveneWordModel {
    pub word_meaning_id: SloveneWordMeaningId,

    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,

    pub categories: Vec<CategoryId>,

    pub translated_at: DateTime<Utc>,

    pub translated_by: Option<UserId>,
}



pub struct InternalEnglishWordMeaningModel {
    pub(crate) word_meaning_id: Uuid,

    pub(crate) disambiguation: Option<String>,

    pub(crate) abbreviation: Option<String>,

    pub(crate) description: Option<String>,

    pub(crate) created_at: DateTime<Utc>,

    pub(crate) last_modified_at: DateTime<Utc>,
}



pub struct EnglishWordMeaningModelWithWeaklyTypedCategoriesAndTranslations {
    pub(crate) word_meaning_id: Uuid,

    pub(crate) disambiguation: Option<String>,

    pub(crate) abbreviation: Option<String>,

    pub(crate) description: Option<String>,

    pub(crate) created_at: DateTime<Utc>,

    pub(crate) last_modified_at: DateTime<Utc>,

    pub(crate) categories: serde_json::Value,

    pub(crate) translates_into: serde_json::Value,
}


impl TryIntoStronglyTypedInternalModel
    for EnglishWordMeaningModelWithWeaklyTypedCategoriesAndTranslations
{
    type InternalModel = InternalEnglishWordMeaningModelWithCategoriesAndTranslations;
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
            Vec<InternalTranslatesIntoSloveneWordModel>,
        >(self.translates_into)
        .map_err(|error| {
            Cow::from(format!(
                "failed to parse returned JSON as internal slovene translations model: {}",
                error
            ))
        })?;


        Ok(Self::InternalModel {
            word_meaning_id: self.word_meaning_id,
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



#[derive(Deserialize)]
pub struct InternalEnglishWordMeaningModelWithCategoriesAndTranslations {
    pub(crate) word_meaning_id: Uuid,

    pub(crate) disambiguation: Option<String>,

    pub(crate) abbreviation: Option<String>,

    pub(crate) description: Option<String>,

    pub(crate) created_at: DateTime<Utc>,

    pub(crate) last_modified_at: DateTime<Utc>,

    pub(crate) categories: Vec<InternalCategoryIdOnlyModel>,

    pub(crate) translates_into: Vec<InternalTranslatesIntoSloveneWordModel>,
}

impl IntoExternalModel for InternalEnglishWordMeaningModelWithCategoriesAndTranslations {
    type ExternalModel = EnglishWordMeaningModelWithCategoriesAndTranslations;

    fn into_external_model(self) -> Self::ExternalModel {
        Self::ExternalModel {
            id: EnglishWordMeaningId::new(self.word_meaning_id),
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



#[derive(Deserialize)]
pub struct InternalCategoryIdOnlyModel {
    pub(crate) category_id: Uuid,
}

impl IntoExternalModel for InternalCategoryIdOnlyModel {
    type ExternalModel = CategoryId;

    fn into_external_model(self) -> Self::ExternalModel {
        CategoryId::new(self.category_id)
    }
}


#[derive(Deserialize)]
pub struct InternalTranslatesIntoSloveneWordModel {
    pub(crate) word_meaning_id: Uuid,

    pub(crate) disambiguation: Option<String>,

    pub(crate) abbreviation: Option<String>,

    pub(crate) description: Option<String>,

    pub(crate) created_at: DateTime<Utc>,

    pub(crate) last_modified_at: DateTime<Utc>,

    pub(crate) categories: Vec<InternalCategoryIdOnlyModel>,

    pub(crate) translated_at: DateTime<Utc>,

    pub(crate) translated_by: Option<Uuid>,
}

impl IntoExternalModel for InternalTranslatesIntoSloveneWordModel {
    type ExternalModel = TranslatesIntoSloveneWordModel;

    fn into_external_model(self) -> Self::ExternalModel {
        Self::ExternalModel {
            word_meaning_id: SloveneWordMeaningId::new(self.word_meaning_id),
            disambiguation: self.disambiguation,
            abbreviation: self.abbreviation,
            description: self.description,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
            categories: self
                .categories
                .into_iter()
                .map(|internal_model| CategoryId::new(internal_model.category_id))
                .collect(),
            translated_at: self.translated_at,
            translated_by: self.translated_by.map(UserId::new),
        }
    }
}


impl IntoExternalModel for InternalEnglishWordMeaningModel {
    type ExternalModel = EnglishWordMeaningModel;

    fn into_external_model(self) -> Self::ExternalModel {
        let id = EnglishWordMeaningId::new(self.word_meaning_id);

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
