use std::borrow::Cow;

use chrono::{DateTime, Utc};
use kolomoni_core::ids::EnglishWordId;
use uuid::Uuid;

use crate::{
    entities::{
        EnglishWordMeaningModelWithCategoriesAndTranslations,
        InternalEnglishWordMeaningModelWithCategoriesAndTranslations,
    },
    IntoExternalModel,
    TryIntoExternalModel,
};




pub struct EnglishWordModel {
    pub word_id: EnglishWordId,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,

    pub lemma: String,
}


pub struct InternalEnglishWordReducedModel {
    pub(crate) word_id: Uuid,

    pub(crate) lemma: String,
}


pub struct InternalEnglishWordModel {
    pub(crate) word_id: Uuid,

    pub(crate) lemma: String,

    pub(crate) created_at: DateTime<Utc>,

    pub(crate) last_modified_at: DateTime<Utc>,
}

impl IntoExternalModel for InternalEnglishWordModel {
    type ExternalModel = EnglishWordModel;

    fn into_external_model(self) -> Self::ExternalModel {
        let word_id = EnglishWordId::new(self.word_id);

        Self::ExternalModel {
            word_id,
            lemma: self.lemma,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
        }
    }
}



pub struct EnglishWordWithMeaningsModel {
    pub word_id: EnglishWordId,

    pub lemma: String,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,

    pub meanings: Vec<EnglishWordMeaningModelWithCategoriesAndTranslations>,
}



pub struct InternalEnglishWordWithMeaningsModel {
    pub(crate) word_id: Uuid,

    pub(crate) lemma: String,

    pub(crate) created_at: DateTime<Utc>,

    pub(crate) last_modified_at: DateTime<Utc>,

    pub(crate) meanings: serde_json::Value,
}


impl TryIntoExternalModel for InternalEnglishWordWithMeaningsModel {
    type ExternalModel = EnglishWordWithMeaningsModel;
    type Error = Cow<'static, str>;

    fn try_into_external_model(self) -> Result<Self::ExternalModel, Self::Error> {
        let internal_meanings = serde_json::from_value::<
            Vec<InternalEnglishWordMeaningModelWithCategoriesAndTranslations>,
        >(self.meanings)
        .map_err(|error| {
            Cow::from(format!(
                "failed to parse returned JSON as internal english word meaning model: {}",
                error
            ))
        })?;

        let meanings = internal_meanings
            .into_iter()
            .map(|internal_meaning| internal_meaning.into_external_model())
            .collect();


        Ok(Self::ExternalModel {
            word_id: EnglishWordId::new(self.word_id),
            lemma: self.lemma,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
            meanings,
        })
    }
}
