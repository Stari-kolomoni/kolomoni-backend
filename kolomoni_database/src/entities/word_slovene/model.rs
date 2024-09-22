use std::borrow::Cow;

use chrono::{DateTime, Utc};
use kolomoni_core::id::SloveneWordId;
use uuid::Uuid;

use crate::{
    entities::{
        InternalSloveneWordMeaningModelWithCategoriesAndTranslations,
        SloveneWordMeaningModelWithCategoriesAndTranslations,
    },
    IntoExternalModel,
    TryIntoExternalModel,
};



pub struct SloveneWordModel {
    pub word_id: SloveneWordId,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,

    pub lemma: String,
}


pub struct InternalSloveneWordReducedModel {
    pub(crate) word_id: Uuid,

    pub(crate) lemma: String,
}



pub struct InternalSloveneWordModel {
    pub(crate) word_id: Uuid,

    pub(crate) lemma: String,

    pub(crate) created_at: DateTime<Utc>,

    pub(crate) last_modified_at: DateTime<Utc>,
}

impl IntoExternalModel for InternalSloveneWordModel {
    type ExternalModel = SloveneWordModel;

    fn into_external_model(self) -> Self::ExternalModel {
        let word_id = SloveneWordId::new(self.word_id);

        Self::ExternalModel {
            word_id,
            lemma: self.lemma,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
        }
    }
}


pub struct SloveneWordWithMeaningsModel {
    pub word_id: SloveneWordId,

    pub lemma: String,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,

    pub meanings: Vec<SloveneWordMeaningModelWithCategoriesAndTranslations>,
}



pub struct InternalSloveneWordWithMeaningsModel {
    pub(crate) word_id: Uuid,

    pub(crate) lemma: String,

    pub(crate) created_at: DateTime<Utc>,

    pub(crate) last_modified_at: DateTime<Utc>,

    pub(crate) meanings: serde_json::Value,
}

impl TryIntoExternalModel for InternalSloveneWordWithMeaningsModel {
    type ExternalModel = SloveneWordWithMeaningsModel;
    type Error = Cow<'static, str>;

    fn try_into_external_model(self) -> Result<Self::ExternalModel, Self::Error> {
        let internal_meanings = serde_json::from_value::<
            Vec<InternalSloveneWordMeaningModelWithCategoriesAndTranslations>,
        >(self.meanings)
        .map_err(|error| {
            Cow::from(format!(
                "failed to parse returned JSON as internal slovene word meaning: {}",
                error
            ))
        })?;

        let meanings = internal_meanings
            .into_iter()
            .map(|internal_meaning| internal_meaning.into_external_model())
            .collect();


        Ok(Self::ExternalModel {
            word_id: SloveneWordId::new(self.word_id),
            lemma: self.lemma,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
            meanings,
        })
    }
}
