use std::borrow::Cow;

use kolomoni_core::ids::WordId;

use crate::{
    deserialize_json_from_value,
    entities::{
        word::{WordLanguage, WordModel},
        word_meaning_slovene::internal::InternalSloveneWordMeaningModelWithDetails,
    },
    IntoExternalModel,
    TryIntoStronglyTypedInternalModel,
};


pub(crate) mod internal_weak {
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    /// A *weak* expanded internal slovene word model, as queried from the database.
    ///
    /// This model is the expanded version of [`InternalSloveneWordModel`],
    /// because it also contains the associated word meanings.
    ///
    /// The normal (or "strong") model this one converts into is [`InternalSloveneWordWithMeaningsModel`].
    /// Upon conversion, the `meanings` field will be deserialized into
    /// [`Vec`]`<`[`InternalSloveneWordMeaningModelWithDetails`]`>`.
    ///
    ///
    /// # Weak models
    /// The phrasing of a model being "weak" in this context refers to the fact that one of the fields
    /// is not fully validated. The usual use-case is a field containing an arbitrary JSON value
    /// ([`serde_json::Value`]), which is then properly deserialized into a strongly-typed structure
    /// when calling the weak model's [`TryIntoStronglyTypedInternalModel`] implementation.
    ///
    ///
    /// [`TryIntoStronglyTypedInternalModel`]: crate::TryIntoStronglyTypedInternalModel
    pub struct WeakInternalSloveneWordWithMeaningsModel {
        pub(crate) word_id: Uuid,

        pub(crate) lemma: String,

        pub(crate) created_at: DateTime<Utc>,

        pub(crate) last_modified_at: DateTime<Utc>,

        pub(crate) meanings: serde_json::Value,
    }
}


pub(crate) mod internal_insert_only {
    use uuid::Uuid;

    /// An insert-only version of the usual internal slovene word model
    /// ([`InternalSloveneWordModel`]), as used for data returns when performing
    /// insert queries on the database. As such, it contains only the word UUID and its lemma.
    pub(crate) struct InsertOnlyInternalSloveneWordModel {
        #[allow(dead_code)]
        pub(crate) word_id: Uuid,

        pub(crate) lemma: String,
    }
}


pub(crate) mod internal {
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    use crate::entities::word_meaning_slovene::internal::InternalSloveneWordMeaningModelWithDetails;


    /// An internal slovene word model, as queried from the database.
    ///
    /// This model does not include the associated meanings, see
    /// [`InternalSloveneWordWithMeaningsModel`] for an expanded version of this model.
    ///
    /// This model contains all the basic fields that we have access to, see
    /// [`InternalSloveneWordReducedModel`] for a reduced version of the model.
    pub(crate) struct InternalSloveneWordModel {
        pub(crate) word_id: Uuid,

        pub(crate) lemma: String,

        pub(crate) created_at: DateTime<Utc>,

        pub(crate) last_modified_at: DateTime<Utc>,
    }


    /// A *weak* expanded internal slovene word model, as queried from the database.
    ///
    /// This model is the expanded version of [`InternalSloveneWordModel`],
    /// because it also contains the associated word meanings.
    ///
    /// Its weak internal model variant is [`WeakInternalSloveneWordWithMeaningsModel`].
    pub(crate) struct InternalSloveneWordWithMeaningsModel {
        pub(crate) word_id: Uuid,

        pub(crate) lemma: String,

        pub(crate) created_at: DateTime<Utc>,

        pub(crate) last_modified_at: DateTime<Utc>,

        pub(crate) meanings: Vec<InternalSloveneWordMeaningModelWithDetails>,
    }
}



mod external {
    use chrono::{DateTime, Utc};
    use kolomoni_core::ids::SloveneWordId;

    use crate::entities::{
        word::WordModel,
        word_meaning_slovene::SloveneWordMeaningModelWithDetails,
    };


    /// A completely bare slovene word model.
    ///
    /// As such, this model contains only the additional fields that slovene words
    /// have in comparison with language-agnostic word models. At the moment,
    /// this is just the word's lemma.
    #[derive(Clone)]
    pub struct BareSloveneWordModel {
        pub lemma: String,
    }


    /// An external slovene word model. Contains the slovene word-specific fields,
    /// such as the lemma, along with the information from the language-agnostic [`WordModel`]
    /// (word UUID, creation and modification timestamp, etc.)
    ///
    ///
    /// # Note
    /// The phrasing "external" in this context refers to a public API that this crate exposes (`pub`-visible models).
    /// Notably, this phrasing *does not* imply that this type is exposed in the sense of the REST API
    /// that the Stari Kolomoni backend server exposes. For those REST API models, see the [`kolomoni_core`] crate.
    #[derive(Clone)]
    pub struct SloveneWordModel {
        base_word: WordModel,

        bare_slovene_word: BareSloveneWordModel,
    }

    impl SloveneWordModel {
        #[inline]
        pub(crate) fn new(bare_word: WordModel, lemma: String) -> Self {
            Self {
                base_word: bare_word,
                bare_slovene_word: BareSloveneWordModel { lemma },
            }
        }

        pub fn id(&self) -> SloveneWordId {
            SloveneWordId::new(self.base_word.id.into_uuid())
        }

        pub fn created_at(&self) -> &DateTime<Utc> {
            &self.base_word.created_at
        }

        pub fn last_modified_at(&self) -> &DateTime<Utc> {
            &self.base_word.last_modified_at
        }

        pub fn lemma(&self) -> &str {
            &self.bare_slovene_word.lemma
        }

        /// Consumes `self` and returns a tuple containing the bare models that make up
        /// an english word:
        /// - the underlying [`WordModel`] (ID, timestamps, ...), and
        /// - the underlying [`BareEnglishWordModel`] (lemma, ...).
        pub fn into_inner(self) -> (WordModel, BareSloveneWordModel) {
            (self.base_word, self.bare_slovene_word)
        }
    }

    impl AsRef<BareSloveneWordModel> for SloveneWordModel {
        fn as_ref(&self) -> &BareSloveneWordModel {
            &self.bare_slovene_word
        }
    }

    impl AsRef<WordModel> for SloveneWordModel {
        fn as_ref(&self) -> &WordModel {
            &self.base_word
        }
    }



    /// An expanded external slovene word model.
    ///
    /// This model is the expanded version of [`SloveneWordModel`],
    /// because it also contains the associated meanings.
    ///
    ///
    /// # Note
    /// The phrasing "external" in this context refers to a public API that this crate exposes (`pub`-visible models).
    /// Notably, this phrasing *does not* imply that this type is exposed in the sense of the REST API
    /// that the Stari Kolomoni backend server exposes. For those REST API models, see the [`kolomoni_core`] crate.
    pub struct SloveneWordWithMeaningsModel {
        base_word: WordModel,

        bare_slovene_word: BareSloveneWordModel,

        meanings: Vec<SloveneWordMeaningModelWithDetails>,
    }

    impl SloveneWordWithMeaningsModel {
        #[inline]
        pub(crate) fn new_with_lemma_and_meanings(
            bare_word: WordModel,
            lemma: String,
            meanings: Vec<SloveneWordMeaningModelWithDetails>,
        ) -> Self {
            Self {
                base_word: bare_word,
                bare_slovene_word: BareSloveneWordModel { lemma },
                meanings,
            }
        }

        pub fn id(&self) -> SloveneWordId {
            SloveneWordId::new(self.base_word.id.into_uuid())
        }

        pub fn created_at(&self) -> &DateTime<Utc> {
            &self.base_word.created_at
        }

        pub fn last_modified_at(&self) -> &DateTime<Utc> {
            &self.base_word.last_modified_at
        }

        pub fn lemma(&self) -> &str {
            &self.bare_slovene_word.lemma
        }

        pub fn meanings(&self) -> &[SloveneWordMeaningModelWithDetails] {
            &self.meanings
        }

        pub fn into_less_detailed_model_and_meanings(
            self,
        ) -> (
            SloveneWordModel,
            Vec<SloveneWordMeaningModelWithDetails>,
        ) {
            (
                SloveneWordModel {
                    base_word: self.base_word,
                    bare_slovene_word: self.bare_slovene_word,
                },
                self.meanings,
            )
        }

        /// Consumes `self` and returns a tuple containing the bare models that make up
        /// an english word:
        /// - the underlying [`WordModel`] (ID, timestamps, ...),
        /// - the underlying [`BareEnglishWordModel`] (lemma, ...), and
        /// - the associated meanings (a [`Vec`] of [`SloveneWordMeaningModelWithDetails`]).
        pub fn into_inner(
            self,
        ) -> (
            WordModel,
            BareSloveneWordModel,
            Vec<SloveneWordMeaningModelWithDetails>,
        ) {
            (
                self.base_word,
                self.bare_slovene_word,
                self.meanings,
            )
        }
    }

    impl AsRef<BareSloveneWordModel> for SloveneWordWithMeaningsModel {
        fn as_ref(&self) -> &BareSloveneWordModel {
            &self.bare_slovene_word
        }
    }

    impl AsRef<WordModel> for SloveneWordWithMeaningsModel {
        fn as_ref(&self) -> &WordModel {
            &self.base_word
        }
    }
}

pub use external::*;



impl IntoExternalModel for internal::InternalSloveneWordModel {
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

impl TryIntoStronglyTypedInternalModel for internal_weak::WeakInternalSloveneWordWithMeaningsModel {
    type InternalModel = internal::InternalSloveneWordWithMeaningsModel;
    type Error = Cow<'static, str>;

    fn try_into_strongly_typed_internal_model(self) -> Result<Self::InternalModel, Self::Error> {
        let internal_meanings = deserialize_json_from_value!(
            self.meanings => Vec<InternalSloveneWordMeaningModelWithDetails>;
            with message: |error| {
                format!(
                    "failed to parse query-returned JSON as internal slovene word meaning model (slovene_word_id={}): {}",
                    self.word_id,
                    error
                )
            }
        )?;


        Ok(Self::InternalModel {
            word_id: self.word_id,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
            lemma: self.lemma,
            meanings: internal_meanings,
        })
    }
}


impl IntoExternalModel for internal::InternalSloveneWordWithMeaningsModel {
    type ExternalModel = SloveneWordWithMeaningsModel;

    fn into_external_model(self) -> Self::ExternalModel {
        let meanings = self
            .meanings
            .into_iter()
            .map(IntoExternalModel::into_external_model)
            .collect();


        Self::ExternalModel::new_with_lemma_and_meanings(
            WordModel {
                id: WordId::new(self.word_id),
                language: WordLanguage::Slovene,
                created_at: self.created_at,
                last_modified_at: self.last_modified_at,
            },
            self.lemma,
            meanings,
        )
    }
}
