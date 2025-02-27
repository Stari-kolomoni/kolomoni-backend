use std::borrow::Cow;

use kolomoni_core::ids::WordId;

use crate::{
    entities::{
        word::{WordLanguage, WordModel},
        word_meaning_english::internal::InternalEnglishWordMeaningModelWithDetails,
    },
    IntoExternalModel,
    TryIntoStronglyTypedInternalModel,
};


pub(crate) mod internal_weak {
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    /// A *weak* expanded internal english word model, as queried from the database.
    ///
    /// This model is the expanded version of [`InternalEnglishWordModel`],
    /// because it also contains the associated word meanings.
    //
    /// The normal (or "strong") model that this one converts into is [`InternalEnglishWordWithMeaningsModel`].
    /// Upon conversion, the `meanings` field will be deserialized into
    /// [`Vec`]`<`[`InternalEnglishWordMeaningModelWithDetails`]`>`.
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
    pub(crate) struct WeakInternalEnglishWordWithMeaningsModel {
        pub(crate) word_id: Uuid,

        pub(crate) created_at: DateTime<Utc>,

        pub(crate) last_modified_at: DateTime<Utc>,

        pub(crate) lemma: String,

        /// Deserialized as [`Vec`]`<`[`InternalEnglishWordMeaningModelWithDetails`]`>` when converting
        /// into a strong internal model, see [`InternalEnglishWordWithMeaningsModel`].
        pub(crate) meanings: serde_json::Value,
    }
}


pub(crate) mod internal_insert_only {
    use uuid::Uuid;

    /// An insert-only version of the usual internal english word model
    /// ([`InternalEnglishWordModel`]), as used for data returns when performing
    /// insert queries on the database. As such, it contains only the word UUID and its lemma.
    pub(crate) struct InsertOnlyInternalEnglishWordModel {
        #[allow(dead_code)]
        pub(crate) word_id: Uuid,

        pub(crate) lemma: String,
    }
}


pub(crate) mod internal {
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    use crate::entities::word_meaning_english::internal::InternalEnglishWordMeaningModelWithDetails;


    /// An internal english word model, as queried from the database.
    ///
    /// This model does not include the associated meanings, see
    /// [`InternalEnglishWordWithMeaningsModel`] for an expanded version of the model.
    pub(crate) struct InternalEnglishWordModel {
        pub(crate) word_id: Uuid,

        pub(crate) created_at: DateTime<Utc>,

        pub(crate) last_modified_at: DateTime<Utc>,

        pub(crate) lemma: String,
    }


    pub(crate) struct InternalEnglishWordWithMeaningsModel {
        pub(crate) word_id: Uuid,

        pub(crate) created_at: DateTime<Utc>,

        pub(crate) last_modified_at: DateTime<Utc>,

        pub(crate) lemma: String,

        pub(crate) meanings: Vec<InternalEnglishWordMeaningModelWithDetails>,
    }
}



mod external {
    use chrono::{DateTime, Utc};
    use kolomoni_core::ids::EnglishWordId;

    use crate::entities::{
        word::WordModel,
        word_meaning_english::EnglishWordMeaningModelWithDetails,
    };

    /// A completely bare english word model.
    ///
    /// As such, this model contains only the additional fields that english words
    /// have in comparison with language-agnostic word models. At the moment,
    /// this is just the word's lemma (the lemma is intentionally not a shared field
    /// to avoid future problems if we decide to get more granular grammatically).
    #[derive(Debug, Clone)]
    pub struct BareEnglishWordModel {
        pub lemma: String,
    }

    /// An external english word model. Contains the english word-specific fields,
    /// such as the lemma, along with the information from the language-agnostic [`WordModel`]
    /// (word UUID, creation and modification timestamp, etc.).
    ///
    ///
    /// # Note
    /// The phrasing "external" in this context refers to a public API that this crate exposes (`pub`-visible models).
    /// Notably, this phrasing *does not* imply that this type is exposed in the sense of the REST API
    /// that the Stari Kolomoni backend server exposes. For those REST API models, see the [`kolomoni_core`] crate.
    #[derive(Debug, Clone)]
    pub struct EnglishWordModel {
        base_word: WordModel,

        bare_english_word: BareEnglishWordModel,
    }

    impl EnglishWordModel {
        #[inline]
        pub(crate) fn new_with_lemma(base_word: WordModel, lemma: String) -> Self {
            Self {
                base_word,
                bare_english_word: BareEnglishWordModel { lemma },
            }
        }

        pub fn id(&self) -> EnglishWordId {
            EnglishWordId::new(self.base_word.id.into_uuid())
        }

        pub fn created_at(&self) -> &DateTime<Utc> {
            &self.base_word.created_at
        }

        pub fn last_modified_at(&self) -> &DateTime<Utc> {
            &self.base_word.last_modified_at
        }

        pub fn lemma(&self) -> &str {
            &self.bare_english_word.lemma
        }

        /// Consumes `self` and returns a tuple containing the bare models that make up
        /// an english word:
        /// - the underlying [`WordModel`] (ID, timestamps, ...), and
        /// - the underlying [`BareEnglishWordModel`] (lemma, ...).
        pub fn into_inner(self) -> (WordModel, BareEnglishWordModel) {
            (self.base_word, self.bare_english_word)
        }
    }

    impl AsRef<BareEnglishWordModel> for EnglishWordModel {
        fn as_ref(&self) -> &BareEnglishWordModel {
            &self.bare_english_word
        }
    }

    impl AsRef<WordModel> for EnglishWordModel {
        fn as_ref(&self) -> &WordModel {
            &self.base_word
        }
    }



    /// An expanded external english word model.
    ///
    /// This model is the expanded version of [`EnglishWordModel`],
    /// because it also contains the associated meanings.
    ///
    ///
    /// # Note
    /// The phrasing "external" in this context refers to a public API that this crate exposes (`pub`-visible models).
    /// Notably, this phrasing *does not* imply that this type is exposed in the sense of the REST API
    /// that the Stari Kolomoni backend server exposes. For those REST API models, see the [`kolomoni_core`] crate.
    pub struct EnglishWordWithMeaningsModel {
        word: WordModel,

        bare_english_word: BareEnglishWordModel,

        meanings: Vec<EnglishWordMeaningModelWithDetails>,
    }

    impl EnglishWordWithMeaningsModel {
        #[inline]
        pub(crate) fn new_with_lemma_and_meanings(
            bare_word: WordModel,
            lemma: String,
            meanings: Vec<EnglishWordMeaningModelWithDetails>,
        ) -> Self {
            Self {
                word: bare_word,
                bare_english_word: BareEnglishWordModel { lemma },
                meanings,
            }
        }

        pub fn id(&self) -> EnglishWordId {
            EnglishWordId::new(self.word.id.into_uuid())
        }

        pub fn created_at(&self) -> &DateTime<Utc> {
            &self.word.created_at
        }

        pub fn last_modified_at(&self) -> &DateTime<Utc> {
            &self.word.last_modified_at
        }

        pub fn lemma(&self) -> &str {
            &self.bare_english_word.lemma
        }

        pub fn meanings(&self) -> &[EnglishWordMeaningModelWithDetails] {
            &self.meanings
        }

        /// Consumes `self` and returns a tuple containing the bare models that make up
        /// an english word:
        /// - the underlying [`WordModel`] (ID, timestamps, ...),
        /// - the underlying [`BareEnglishWordModel`] (lemma, ...), and
        /// - the linked word meanings ([`Vec`] of [`EnglishWordMeaningModelWithDetails`]).
        pub fn into_inner(
            self,
        ) -> (
            WordModel,
            BareEnglishWordModel,
            Vec<EnglishWordMeaningModelWithDetails>,
        ) {
            (self.word, self.bare_english_word, self.meanings)
        }
    }

    impl AsRef<BareEnglishWordModel> for EnglishWordWithMeaningsModel {
        fn as_ref(&self) -> &BareEnglishWordModel {
            &self.bare_english_word
        }
    }

    impl AsRef<WordModel> for EnglishWordWithMeaningsModel {
        fn as_ref(&self) -> &WordModel {
            &self.word
        }
    }
}

pub use external::*;




impl TryIntoStronglyTypedInternalModel for internal_weak::WeakInternalEnglishWordWithMeaningsModel {
    type InternalModel = internal::InternalEnglishWordWithMeaningsModel;
    type Error = Cow<'static, str>;

    fn try_into_strongly_typed_internal_model(self) -> Result<Self::InternalModel, Self::Error> {
        let meanings =
            serde_json::from_value::<Vec<InternalEnglishWordMeaningModelWithDetails>>(self.meanings)
                .map_err(|error| {
                    Cow::from(format!(
                        "failed to parse returned JSON as internal english word meaning model: {}",
                        error
                    ))
                })?;


        Ok(Self::InternalModel {
            word_id: self.word_id,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
            lemma: self.lemma,
            meanings,
        })
    }
}


impl IntoExternalModel for internal::InternalEnglishWordModel {
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


impl IntoExternalModel for internal::InternalEnglishWordWithMeaningsModel {
    type ExternalModel = EnglishWordWithMeaningsModel;

    fn into_external_model(self) -> Self::ExternalModel {
        let meanings = self
            .meanings
            .into_iter()
            .map(|meaning| meaning.into_external_model())
            .collect();

        Self::ExternalModel::new_with_lemma_and_meanings(
            WordModel {
                id: WordId::new(self.word_id),
                language: WordLanguage::English,
                created_at: self.created_at,
                last_modified_at: self.last_modified_at,
            },
            self.lemma,
            meanings,
        )
    }
}
