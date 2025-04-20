use std::borrow::Cow;

use kolomoni_core::ids::{CategoryId, UserId, WordId, WordMeaningId};
use uuid::Uuid;

use crate::{
    entities::{
        word::{WordLanguage, WordModel},
        word_meaning::WordMeaningModel,
        word_meaning_slovene::{BareSloveneWordMeaningModel, SloveneWordMeaningModel},
        word_slovene::SloveneWordModel,
    },
    IntoExternalModel,
    TryIntoStronglyTypedInternalModel,
};


pub(crate) mod internal_weak {
    use chrono::{DateTime, Utc};
    use serde::Deserialize;
    use uuid::Uuid;


    /// A *weak* expanded internal english word meaning model, as queried from the database.
    ///
    /// This model is an expanded version of [`InternalEnglishWordMeaningModel`],
    /// because it also contains the associated categories and translations.
    ///
    /// The normal (or "strong") model that this one converts into is [`InternalEnglishWordMeaningModelWithDetails`].
    /// Upon conversion, the `categories` field will be deserialized into [`Vec`]`<`[`Uuid`]`>`,
    /// and the `translations` field will be deserialized into [`Vec`]`<`[`InternalSloveneTranslationModel`]`>`.
    ///
    ///
    /// # Weak models
    /// The phrasing of a model being "weak" in this context refers to the fact that one of the fields
    /// is not fully validated. The usual use-case is a field containing an arbitrary JSON value
    /// ([`serde_json::Value`]), which is then properly deserialized into a strongly-typed structure
    /// when calling the weak model's [`TryIntoStronglyTypedInternalModel`] implementation.
    ///
    ///
    /// [`InternalSloveneTranslationModel`]: super::internal::InternalSloveneTranslationModel
    /// [`TryIntoStronglyTypedInternalModel`]: crate::TryIntoStronglyTypedInternalModel
    #[derive(Debug, Deserialize)]
    pub(crate) struct WeakInternalEnglishWordMeaningModelWithDetails {
        pub(crate) word_id: Uuid,

        pub(crate) word_meaning_id: Uuid,

        pub(crate) created_at: DateTime<Utc>,

        pub(crate) last_modified_at: DateTime<Utc>,

        pub(crate) disambiguation: Option<String>,

        pub(crate) abbreviation: Option<String>,

        pub(crate) description: Option<String>,

        pub(crate) categories: serde_json::Value,

        pub(crate) translations: serde_json::Value,
    }
}


pub(crate) mod internal_insert_only {
    use uuid::Uuid;

    /// An insert-only internal english word meaning model, as used for
    /// data returns when performing insert queries on the database.
    #[derive(Debug)]
    pub(crate) struct InsertOnlyInternalEnglishWordMeaningModel {
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


    /// An internal english word meaning model, as queried from the database.
    ///
    /// This model does not include the linked categories and translations, see
    /// [`InternalEnglishWordMeaningModelWithDetails`] an expanded model.
    // TODO add info about parent word (into SQL query as well)
    #[derive(Debug, Deserialize)]
    pub(crate) struct InternalEnglishWordMeaningModel {
        pub(crate) word_id: Uuid,

        pub(crate) word_meaning_id: Uuid,

        pub(crate) created_at: DateTime<Utc>,

        pub(crate) last_modified_at: DateTime<Utc>,

        pub(crate) disambiguation: Option<String>,

        pub(crate) abbreviation: Option<String>,

        pub(crate) description: Option<String>,
    }



    #[derive(Debug, Deserialize)]
    pub(crate) struct InternalEnglishWordMeaningModelWithDetails {
        pub(crate) word_id: Uuid,

        pub(crate) word_meaning_id: Uuid,

        pub(crate) created_at: DateTime<Utc>,

        pub(crate) last_modified_at: DateTime<Utc>,

        pub(crate) disambiguation: Option<String>,

        pub(crate) abbreviation: Option<String>,

        pub(crate) description: Option<String>,

        pub(crate) categories: Vec<Uuid>,

        pub(crate) translations: Vec<InternalSloveneTranslationModel>,
    }



    #[derive(Debug, Deserialize)]
    pub(crate) struct InternalTranslatedSloveneWordModel {
        pub(crate) word_id: Uuid,

        pub(crate) created_at: DateTime<Utc>,

        pub(crate) last_modified_at: DateTime<Utc>,

        pub(crate) lemma: String,
    }

    #[derive(Debug, Deserialize)]
    pub(crate) struct InternalTranslatedSloveneMeaningModel {
        pub(crate) word_id: Uuid,

        pub(crate) word_meaning_id: Uuid,

        pub(crate) created_at: DateTime<Utc>,

        pub(crate) last_modified_at: DateTime<Utc>,

        pub(crate) disambiguation: Option<String>,

        pub(crate) abbreviation: Option<String>,

        pub(crate) description: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    pub(crate) struct InternalSloveneTranslationModel {
        pub(crate) word: InternalTranslatedSloveneWordModel,

        pub(crate) word_meaning: InternalTranslatedSloveneMeaningModel,

        pub(crate) translated_at: DateTime<Utc>,

        pub(crate) translated_by: Option<Uuid>,
    }
}



mod external {
    use chrono::{DateTime, Utc};
    use kolomoni_core::ids::{CategoryId, EnglishWordId, EnglishWordMeaningId, UserId};

    use crate::entities::{
        word_meaning::WordMeaningModel,
        word_meaning_slovene::SloveneWordMeaningModel,
        word_slovene::SloveneWordModel,
    };


    #[derive(Clone)]
    pub struct BareEnglishWordMeaningModel {
        pub disambiguation: Option<String>,

        pub abbreviation: Option<String>,

        pub description: Option<String>,
    }


    /// An external english word meaning model (disambiguation, description, abbreviation, etc.), along
    /// with the information from the base word meaning model (meaning ID, creation and modification timestamps, etc.).
    ///
    /// For an expanded version that contains information about linked categories and
    /// translations, see [`EnglishWordMeaningModelWithDetails`].
    ///
    ///
    /// # Note
    /// The phrasing "external" in this context refers to a public API that this crate exposes (`pub`-visible models).
    /// Notably, this phrasing *does not* imply that this type is exposed in the sense of the REST API
    /// that the Stari Kolomoni backend server exposes. For those REST API models, see the [`kolomoni_core`] crate.
    #[derive(Clone)]
    pub struct EnglishWordMeaningModel {
        word_meaning: WordMeaningModel,

        bare_english_word_meaning: BareEnglishWordMeaningModel,
    }

    impl EnglishWordMeaningModel {
        #[inline]
        pub(crate) fn new(
            word_meaning: WordMeaningModel,
            bare_english_word_meaning: BareEnglishWordMeaningModel,
        ) -> Self {
            Self {
                word_meaning,
                bare_english_word_meaning,
            }
        }

        pub fn id(&self) -> EnglishWordMeaningId {
            self.word_meaning
                .word_meaning_id
                .to_english_word_meaning_id_unchecked()
        }

        pub fn parent_word_id(&self) -> EnglishWordId {
            EnglishWordId::new(self.word_meaning.word_id.into_uuid())
        }

        pub fn description(&self) -> Option<&str> {
            self.bare_english_word_meaning.description.as_deref()
        }

        pub fn disambiguation(&self) -> Option<&str> {
            self.bare_english_word_meaning.disambiguation.as_deref()
        }

        pub fn abbreviation(&self) -> Option<&str> {
            self.bare_english_word_meaning.abbreviation.as_deref()
        }

        pub fn into_inner(self) -> (WordMeaningModel, BareEnglishWordMeaningModel) {
            (self.word_meaning, self.bare_english_word_meaning)
        }
    }

    impl AsRef<WordMeaningModel> for EnglishWordMeaningModel {
        fn as_ref(&self) -> &WordMeaningModel {
            &self.word_meaning
        }
    }

    impl AsRef<BareEnglishWordMeaningModel> for EnglishWordMeaningModel {
        fn as_ref(&self) -> &BareEnglishWordMeaningModel {
            &self.bare_english_word_meaning
        }
    }



    pub struct EnglishWordMeaningModelWithShallowDetails {
        word_meaning: WordMeaningModel,

        bare_english_word_meaning: BareEnglishWordMeaningModel,

        categories: Vec<CategoryId>,
    }

    impl EnglishWordMeaningModelWithShallowDetails {
        #[inline]
        pub fn new(
            word_meaning: WordMeaningModel,
            english_word_meaning: BareEnglishWordMeaningModel,
            categories: Vec<CategoryId>,
        ) -> Self {
            Self {
                word_meaning,
                bare_english_word_meaning: english_word_meaning,
                categories,
            }
        }

        pub fn id(&self) -> EnglishWordMeaningId {
            self.word_meaning
                .word_meaning_id
                .to_english_word_meaning_id_unchecked()
        }

        pub fn word_id(&self) -> EnglishWordId {
            EnglishWordId::new(self.word_meaning.word_id.into_uuid())
        }

        pub fn description(&self) -> Option<&str> {
            self.bare_english_word_meaning.description.as_deref()
        }

        pub fn disambiguation(&self) -> Option<&str> {
            self.bare_english_word_meaning.disambiguation.as_deref()
        }

        pub fn abbreviation(&self) -> Option<&str> {
            self.bare_english_word_meaning.abbreviation.as_deref()
        }

        pub fn categories(&self) -> &[CategoryId] {
            &self.categories
        }

        pub fn into_inner(
            self,
        ) -> (
            WordMeaningModel,
            BareEnglishWordMeaningModel,
            Vec<CategoryId>,
        ) {
            (
                self.word_meaning,
                self.bare_english_word_meaning,
                self.categories,
            )
        }
    }



    /// An expanded external english word meaning model that includes information about
    /// linked categories and translations.
    ///
    /// This model is an expanded version of [`EnglishWordMeaningModel`].
    ///
    /// # Note
    /// External in this context (this crate) does not mean external in the sense of the REST API
    /// that this project exposes, but in the sense of being public in this crate, [`kolomoni_database`][crate].
    pub struct EnglishWordMeaningModelWithDetails {
        word_meaning: WordMeaningModel,

        bare_english_word_meaning: BareEnglishWordMeaningModel,

        categories: Vec<CategoryId>,

        translations: Vec<SloveneTranslationModel>,
    }

    impl EnglishWordMeaningModelWithDetails {
        #[inline]
        pub fn new(
            word_meaning: WordMeaningModel,
            bare_english_word_meaning: BareEnglishWordMeaningModel,
            categories: Vec<CategoryId>,
            translations: Vec<SloveneTranslationModel>,
        ) -> Self {
            Self {
                word_meaning,
                bare_english_word_meaning,
                categories,
                translations,
            }
        }

        pub fn id(&self) -> EnglishWordMeaningId {
            self.word_meaning
                .word_meaning_id
                .to_english_word_meaning_id_unchecked()
        }

        pub fn parent_word_id(&self) -> EnglishWordId {
            self.word_meaning.word_id.to_english_word_id_unchecked()
        }

        pub fn description(&self) -> Option<&str> {
            self.bare_english_word_meaning.description.as_deref()
        }

        pub fn disambiguation(&self) -> Option<&str> {
            self.bare_english_word_meaning.disambiguation.as_deref()
        }

        pub fn abbreviation(&self) -> Option<&str> {
            self.bare_english_word_meaning.abbreviation.as_deref()
        }

        pub fn categories(&self) -> &[CategoryId] {
            &self.categories
        }

        pub fn translations(&self) -> &[SloveneTranslationModel] {
            &self.translations
        }

        pub fn into_less_detailed_model_and_individual_details(
            self,
        ) -> (
            EnglishWordMeaningModel,
            Vec<CategoryId>,
            Vec<SloveneTranslationModel>,
        ) {
            (
                EnglishWordMeaningModel {
                    word_meaning: self.word_meaning,
                    bare_english_word_meaning: self.bare_english_word_meaning,
                },
                self.categories,
                self.translations,
            )
        }

        pub fn into_inner(
            self,
        ) -> (
            WordMeaningModel,
            BareEnglishWordMeaningModel,
            Vec<CategoryId>,
            Vec<SloveneTranslationModel>,
        ) {
            (
                self.word_meaning,
                self.bare_english_word_meaning,
                self.categories,
                self.translations,
            )
        }
    }

    impl AsRef<WordMeaningModel> for EnglishWordMeaningModelWithDetails {
        fn as_ref(&self) -> &WordMeaningModel {
            &self.word_meaning
        }
    }

    impl AsRef<BareEnglishWordMeaningModel> for EnglishWordMeaningModelWithDetails {
        fn as_ref(&self) -> &BareEnglishWordMeaningModel {
            &self.bare_english_word_meaning
        }
    }


    /// An external (partial) slovene word meaning model, as it appears in
    /// [`EnglishWordMeaningModelWithDetails::translations`].
    ///
    /// To put it differently, this type describes a slovene word meaning, but with associated word context
    /// and without the additional details, such as what translations and categories for the slovene translation itself.
    pub struct SloveneTranslationModel {
        pub word: SloveneWordModel,

        pub word_meaning: SloveneWordMeaningModel,

        pub translated_at: DateTime<Utc>,

        pub translated_by: Option<UserId>,
    }
}

pub use external::*;



impl IntoExternalModel for internal::InternalSloveneTranslationModel {
    type ExternalModel = SloveneTranslationModel;

    fn into_external_model(self) -> Self::ExternalModel {
        Self::ExternalModel {
            word: self.word.into_external_model(),
            word_meaning: self.word_meaning.into_external_model(),
            translated_at: self.translated_at,
            translated_by: self.translated_by.map(UserId::new),
        }
    }
}


impl IntoExternalModel for internal::InternalEnglishWordMeaningModel {
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
                disambiguation: self.disambiguation,
                abbreviation: self.abbreviation,
                description: self.description,
            },
        )
    }
}


impl TryIntoStronglyTypedInternalModel
    for internal_weak::WeakInternalEnglishWordMeaningModelWithDetails
{
    type InternalModel = internal::InternalEnglishWordMeaningModelWithDetails;
    type Error = Cow<'static, str>;

    fn try_into_strongly_typed_internal_model(self) -> Result<Self::InternalModel, Self::Error> {
        let categories = serde_json::from_value::<Vec<Uuid>>(self.categories)
            .map_err(|error| {
                Cow::from(format!(
                    "failed to parse returned JSON as internal ID-only categories model: {}",
                    error
                ))
            })?
            .into_iter()
            .collect();

        let translations = serde_json::from_value::<Vec<internal::InternalSloveneTranslationModel>>(
            self.translations,
        )
        .map_err(|error| {
            Cow::from(format!(
                "failed to parse returned JSON as internal slovene translations model: {}",
                error
            ))
        })?
        .into_iter()
        .collect();


        Ok(Self::InternalModel {
            word_id: self.word_id,
            word_meaning_id: self.word_meaning_id,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
            abbreviation: self.abbreviation,
            disambiguation: self.disambiguation,
            description: self.description,
            categories,
            translations,
        })
    }
}


impl IntoExternalModel for internal::InternalEnglishWordMeaningModelWithDetails {
    type ExternalModel = EnglishWordMeaningModelWithDetails;

    fn into_external_model(self) -> Self::ExternalModel {
        let categories = self.categories.into_iter().map(CategoryId::new).collect();

        let translations = self
            .translations
            .into_iter()
            .map(|translation| translation.into_external_model())
            .collect();


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
            categories,
            translations,
        )
    }
}



impl IntoExternalModel for internal::InternalTranslatedSloveneWordModel {
    type ExternalModel = SloveneWordModel;

    fn into_external_model(self) -> Self::ExternalModel {
        Self::ExternalModel::new(
            WordModel {
                id: WordId::new(self.word_id),
                language: WordLanguage::Slovene,
                created_at: self.created_at,
                last_modified_at: self.last_modified_at,
            },
            self.lemma,
        )
    }
}

impl IntoExternalModel for internal::InternalTranslatedSloveneMeaningModel {
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
