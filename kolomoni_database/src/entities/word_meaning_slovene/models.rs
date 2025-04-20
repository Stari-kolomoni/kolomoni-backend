use std::borrow::Cow;

use internal::InternalEnglishTranslationModel;
use kolomoni_core::ids::{CategoryId, UserId, WordId, WordMeaningId};
use uuid::Uuid;

use crate::{
    entities::{
        word::{WordLanguage, WordModel},
        word_english::EnglishWordModel,
        word_meaning::WordMeaningModel,
        word_meaning_english::{BareEnglishWordMeaningModel, EnglishWordMeaningModel},
    },
    IntoExternalModel,
    TryIntoStronglyTypedInternalModel,
};


pub(crate) mod internal_weak {
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    pub(crate) struct WeakInternalSloveneWordMeaningModelWithDetails {
        pub(crate) word_id: Uuid,

        pub(crate) word_meaning_id: Uuid,

        pub(crate) disambiguation: Option<String>,

        pub(crate) abbreviation: Option<String>,

        pub(crate) description: Option<String>,

        pub(crate) created_at: DateTime<Utc>,

        pub(crate) last_modified_at: DateTime<Utc>,

        pub(crate) categories: serde_json::Value,

        pub(crate) translations: serde_json::Value,
    }
}

pub(crate) mod internal_insert_only {
    use uuid::Uuid;

    /// An insert-only version of the usual internal slovene word meaning model
    /// ([`InternalSloveneWordMeaningModel`]), as used for data returns when performing
    /// insert queries on the database.
    #[derive(Debug)]
    pub(crate) struct InsertOnlyInternalSloveneWordMeaningModel {
        #[allow(dead_code)]
        pub(crate) word_meaning_id: Uuid,

        pub(crate) disambiguation: Option<String>,

        pub(crate) abbreviation: Option<String>,

        pub(crate) description: Option<String>,
    }
}


pub(crate) mod internal {
    use chrono::{DateTime, Utc};
    use serde::Deserialize;
    use uuid::Uuid;


    #[derive(Debug, Deserialize)]
    pub(crate) struct InternalSloveneWordMeaningModel {
        pub(crate) word_id: Uuid,

        pub(crate) word_meaning_id: Uuid,

        pub(crate) disambiguation: Option<String>,

        pub(crate) abbreviation: Option<String>,

        pub(crate) description: Option<String>,

        pub(crate) created_at: DateTime<Utc>,

        pub(crate) last_modified_at: DateTime<Utc>,
    }

    #[derive(Debug, Deserialize)]
    pub(crate) struct InternalSloveneWordMeaningModelWithDetails {
        pub(crate) word_id: Uuid,

        pub(crate) word_meaning_id: Uuid,

        pub(crate) disambiguation: Option<String>,

        pub(crate) abbreviation: Option<String>,

        pub(crate) description: Option<String>,

        pub(crate) created_at: DateTime<Utc>,

        pub(crate) last_modified_at: DateTime<Utc>,

        pub(crate) categories: Vec<Uuid>,

        pub(crate) translations: Vec<InternalEnglishTranslationModel>,
    }



    #[derive(Debug, Deserialize)]
    pub(crate) struct InternalTranslatedEnglishWordModel {
        pub(crate) word_id: Uuid,

        pub(crate) created_at: DateTime<Utc>,

        pub(crate) last_modified_at: DateTime<Utc>,

        pub(crate) lemma: String,
    }

    #[derive(Debug, Deserialize)]
    pub(crate) struct InternalTranslatedEnglishMeaningModel {
        pub(crate) word_id: Uuid,

        pub(crate) word_meaning_id: Uuid,

        pub(crate) created_at: DateTime<Utc>,

        pub(crate) last_modified_at: DateTime<Utc>,

        pub(crate) disambiguation: Option<String>,

        pub(crate) abbreviation: Option<String>,

        pub(crate) description: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    pub(crate) struct InternalEnglishTranslationModel {
        pub(crate) word: InternalTranslatedEnglishWordModel,

        pub(crate) word_meaning: InternalTranslatedEnglishMeaningModel,

        pub(crate) translated_at: DateTime<Utc>,

        pub(crate) translated_by: Option<Uuid>,
    }
}



mod external {
    use chrono::{DateTime, Utc};
    use kolomoni_core::ids::{CategoryId, SloveneWordId, SloveneWordMeaningId, UserId};

    use crate::entities::{
        word_english::EnglishWordModel,
        word_meaning::WordMeaningModel,
        word_meaning_english::EnglishWordMeaningModel,
    };


    #[derive(Clone)]
    pub struct BareSloveneWordMeaningModel {
        pub disambiguation: Option<String>,

        pub abbreviation: Option<String>,

        pub description: Option<String>,
    }


    #[derive(Clone)]
    pub struct SloveneWordMeaningModel {
        word_meaning: WordMeaningModel,

        bare_slovene_word_meaning: BareSloveneWordMeaningModel,
    }

    impl SloveneWordMeaningModel {
        #[inline]
        pub(crate) fn new(
            word_meaning: WordMeaningModel,
            slovene_word_meaning: BareSloveneWordMeaningModel,
        ) -> Self {
            Self {
                word_meaning,
                bare_slovene_word_meaning: slovene_word_meaning,
            }
        }

        pub fn id(&self) -> SloveneWordMeaningId {
            self.word_meaning
                .word_meaning_id
                .to_slovene_word_meaning_id_unchecked()
        }

        pub fn parent_word_id(&self) -> SloveneWordId {
            SloveneWordId::new(self.word_meaning.word_id.into_uuid())
        }


        pub fn description(&self) -> Option<&str> {
            self.bare_slovene_word_meaning.description.as_deref()
        }

        pub fn disambiguation(&self) -> Option<&str> {
            self.bare_slovene_word_meaning.disambiguation.as_deref()
        }

        pub fn abbreviation(&self) -> Option<&str> {
            self.bare_slovene_word_meaning.abbreviation.as_deref()
        }

        pub fn into_inner(self) -> (WordMeaningModel, BareSloveneWordMeaningModel) {
            (self.word_meaning, self.bare_slovene_word_meaning)
        }
    }

    impl AsRef<WordMeaningModel> for SloveneWordMeaningModel {
        fn as_ref(&self) -> &WordMeaningModel {
            &self.word_meaning
        }
    }

    impl AsRef<BareSloveneWordMeaningModel> for SloveneWordMeaningModel {
        fn as_ref(&self) -> &BareSloveneWordMeaningModel {
            &self.bare_slovene_word_meaning
        }
    }



    pub struct SloveneWordMeaningModelWithShallowDetails {
        word_meaning: WordMeaningModel,

        bare_slovene_word_meaning: BareSloveneWordMeaningModel,

        categories: Vec<CategoryId>,
    }

    impl SloveneWordMeaningModelWithShallowDetails {
        #[inline]
        pub(crate) fn new(
            word_meaning: WordMeaningModel,
            slovene_word_meaning: BareSloveneWordMeaningModel,
            categories: Vec<CategoryId>,
        ) -> Self {
            Self {
                word_meaning,
                bare_slovene_word_meaning: slovene_word_meaning,
                categories,
            }
        }

        pub fn id(&self) -> SloveneWordMeaningId {
            self.word_meaning
                .word_meaning_id
                .to_slovene_word_meaning_id_unchecked()
        }

        pub fn word_id(&self) -> SloveneWordId {
            SloveneWordId::new(self.word_meaning.word_id.into_uuid())
        }


        pub fn description(&self) -> Option<&str> {
            self.bare_slovene_word_meaning.description.as_deref()
        }

        pub fn disambiguation(&self) -> Option<&str> {
            self.bare_slovene_word_meaning.disambiguation.as_deref()
        }

        pub fn abbreviation(&self) -> Option<&str> {
            self.bare_slovene_word_meaning.abbreviation.as_deref()
        }

        pub fn categories(&self) -> &[CategoryId] {
            &self.categories
        }


        pub fn into_inner(
            self,
        ) -> (
            WordMeaningModel,
            BareSloveneWordMeaningModel,
            Vec<CategoryId>,
        ) {
            (
                self.word_meaning,
                self.bare_slovene_word_meaning,
                self.categories,
            )
        }
    }



    pub struct SloveneWordMeaningModelWithDetails {
        word_meaning: WordMeaningModel,

        bare_slovene_word_meaning: BareSloveneWordMeaningModel,

        categories: Vec<CategoryId>,

        translations: Vec<EnglishTranslationModel>,
    }

    impl SloveneWordMeaningModelWithDetails {
        #[inline]
        pub(crate) fn new(
            word_meaning: WordMeaningModel,
            bare_slovene_word_meaning: BareSloveneWordMeaningModel,
            categories: Vec<CategoryId>,
            translations: Vec<EnglishTranslationModel>,
        ) -> Self {
            Self {
                word_meaning,
                bare_slovene_word_meaning,
                categories,
                translations,
            }
        }

        pub fn id(&self) -> SloveneWordMeaningId {
            self.word_meaning
                .word_meaning_id
                .to_slovene_word_meaning_id_unchecked()
        }

        pub fn parent_word_id(&self) -> SloveneWordId {
            self.word_meaning.word_id.to_slovene_word_id_unchecked()
        }

        pub fn description(&self) -> Option<&str> {
            self.bare_slovene_word_meaning.description.as_deref()
        }

        pub fn disambiguation(&self) -> Option<&str> {
            self.bare_slovene_word_meaning.disambiguation.as_deref()
        }

        pub fn abbreviation(&self) -> Option<&str> {
            self.bare_slovene_word_meaning.abbreviation.as_deref()
        }

        pub fn categories(&self) -> &[CategoryId] {
            &self.categories
        }

        pub fn translations(&self) -> &[EnglishTranslationModel] {
            &self.translations
        }


        pub fn into_less_detailed_model_and_individual_details(
            self,
        ) -> (
            SloveneWordMeaningModel,
            Vec<CategoryId>,
            Vec<EnglishTranslationModel>,
        ) {
            (
                SloveneWordMeaningModel {
                    word_meaning: self.word_meaning,
                    bare_slovene_word_meaning: self.bare_slovene_word_meaning,
                },
                self.categories,
                self.translations,
            )
        }

        pub fn into_inner(
            self,
        ) -> (
            WordMeaningModel,
            BareSloveneWordMeaningModel,
            Vec<CategoryId>,
            Vec<EnglishTranslationModel>,
        ) {
            (
                self.word_meaning,
                self.bare_slovene_word_meaning,
                self.categories,
                self.translations,
            )
        }
    }

    impl AsRef<WordMeaningModel> for SloveneWordMeaningModelWithShallowDetails {
        fn as_ref(&self) -> &WordMeaningModel {
            &self.word_meaning
        }
    }

    impl AsRef<BareSloveneWordMeaningModel> for SloveneWordMeaningModelWithShallowDetails {
        fn as_ref(&self) -> &BareSloveneWordMeaningModel {
            &self.bare_slovene_word_meaning
        }
    }



    pub struct EnglishTranslationModel {
        pub word: EnglishWordModel,

        pub word_meaning: EnglishWordMeaningModel,

        pub translated_at: DateTime<Utc>,

        pub translated_by: Option<UserId>,
    }
}

pub use external::*;




impl TryIntoStronglyTypedInternalModel
    for internal_weak::WeakInternalSloveneWordMeaningModelWithDetails
{
    type InternalModel = internal::InternalSloveneWordMeaningModelWithDetails;
    type Error = Cow<'static, str>;

    fn try_into_strongly_typed_internal_model(self) -> Result<Self::InternalModel, Self::Error> {
        let categories = serde_json::from_value::<Vec<Uuid>>(self.categories).map_err(|error| {
            Cow::from(format!(
                "failed to parse returned JSON as internal ID-only categories model: {}",
                error
            ))
        })?;

        let translations =
            serde_json::from_value::<Vec<InternalEnglishTranslationModel>>(self.translations)
                .map_err(|error| {
                    Cow::from(format!(
                        "failed to parse returned JSON as internal english translations model: {}",
                        error
                    ))
                })?;


        Ok(Self::InternalModel {
            word_id: self.word_id,
            word_meaning_id: self.word_meaning_id,
            disambiguation: self.disambiguation,
            abbreviation: self.abbreviation,
            description: self.description,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
            categories,
            translations,
        })
    }
}


impl IntoExternalModel for internal::InternalSloveneWordMeaningModel {
    type ExternalModel = SloveneWordMeaningModel;

    fn into_external_model(self) -> Self::ExternalModel {
        Self::ExternalModel::new(
            WordMeaningModel {
                word_id: WordId::new(self.word_id),
                word_meaning_id: WordMeaningId::new(self.word_meaning_id),
                created_at: self.created_at,
                last_modified_at: self.last_modified_at,
            },
            BareSloveneWordMeaningModel {
                description: self.description,
                abbreviation: self.abbreviation,
                disambiguation: self.disambiguation,
            },
        )
    }
}


impl IntoExternalModel for internal::InternalSloveneWordMeaningModelWithDetails {
    type ExternalModel = SloveneWordMeaningModelWithDetails;

    fn into_external_model(self) -> Self::ExternalModel {
        let categories = self.categories.into_iter().map(CategoryId::new).collect();

        let translations = self
            .translations
            .into_iter()
            .map(IntoExternalModel::into_external_model)
            .collect();


        Self::ExternalModel::new(
            WordMeaningModel {
                word_id: WordId::new(self.word_id),
                word_meaning_id: WordMeaningId::new(self.word_meaning_id),
                created_at: self.created_at,
                last_modified_at: self.last_modified_at,
            },
            BareSloveneWordMeaningModel {
                description: self.description,
                abbreviation: self.abbreviation,
                disambiguation: self.disambiguation,
            },
            categories,
            translations,
        )
    }
}



impl IntoExternalModel for InternalEnglishTranslationModel {
    type ExternalModel = EnglishTranslationModel;

    fn into_external_model(self) -> Self::ExternalModel {
        Self::ExternalModel {
            word: self.word.into_external_model(),
            word_meaning: self.word_meaning.into_external_model(),
            translated_at: self.translated_at,
            translated_by: self.translated_by.map(UserId::new),
        }
    }
}

impl IntoExternalModel for internal::InternalTranslatedEnglishWordModel {
    type ExternalModel = EnglishWordModel;

    fn into_external_model(self) -> Self::ExternalModel {
        Self::ExternalModel::new_with_lemma(
            WordModel {
                id: WordId::new(self.word_id),
                language: WordLanguage::English,
                created_at: self.created_at,
                last_modified_at: self.last_modified_at,
            },
            self.lemma,
        )
    }
}

impl IntoExternalModel for internal::InternalTranslatedEnglishMeaningModel {
    type ExternalModel = EnglishWordMeaningModel;

    fn into_external_model(self) -> Self::ExternalModel {
        Self::ExternalModel::new(
            WordMeaningModel {
                word_id: WordId::new(self.word_id),
                word_meaning_id: WordMeaningId::new(self.word_meaning_id),
                created_at: self.created_at,
                last_modified_at: self.last_modified_at,
            },
            BareEnglishWordMeaningModel {
                description: self.description,
                abbreviation: self.abbreviation,
                disambiguation: self.disambiguation,
            },
        )
    }
}
