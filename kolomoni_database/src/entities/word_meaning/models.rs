use kolomoni_core::ids::{WordId, WordMeaningId};

use crate::IntoExternalModel;



pub(crate) mod internal {
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    /// An internal language-agnostic word meaning model, as queried from the database.
    /// Contains the fields that all word meaning models share, which are currently:
    /// UUID and the creation and modification timestamp.
    ///
    /// For language-specific internal models, see the [`word_meaning_english`] and
    /// [`word_meaning_slovene`] modules (e.g. [`InternalEnglishWordMeaningModel`],
    /// [`InternalSloveneWordMeaningModel`], etc.).
    ///
    ///
    /// [`word_meaning_english`]: crate::entities::word_meaning_english
    /// [`word_meaning_slovene`]: crate::entities::word_meaning_slovene
    /// [`InternalEnglishWordMeaningModel`]: crate::entities::InternalEnglishWordMeaningModel
    /// [`InternalSloveneWordMeaningModel`]: crate::entities::InternalSloveneWordMeaningModel
    pub(crate) struct InternalWordMeaningModel {
        pub(crate) id: Uuid,

        pub(crate) word_id: Uuid,

        pub(crate) created_at: DateTime<Utc>,

        pub(crate) last_modified_at: DateTime<Utc>,
    }
}



mod external {
    use chrono::{DateTime, Utc};
    use kolomoni_core::ids::{WordId, WordMeaningId};

    /// An external language-agnostic word model. Contains the fields that all word meaning
    /// models share, which are currently: UUID, creation and modification timestamps.
    ///
    /// For language-specific external models, see the [`word_meaning_english`] and
    /// [`word_meaning_slovene`] modules (e.g. [`EnglishWordMeaningModel`],
    /// [`SloveneWordMeaningModel`], etc.).
    ///
    ///
    /// # Note
    /// The phrasing "external" in this context refers to a public API that this crate exposes (`pub`-visible models).
    /// Notably, this phrasing *does not* imply that this type is exposed in the sense of the REST API
    /// that the Stari Kolomoni backend server exposes. For those REST API models, see the [`kolomoni_core`] crate.
    ///
    ///
    /// [`word_meaning_english`]: crate::entities::word_meaning_english
    /// [`word_meaning_slovene`]: crate::entities::word_meaning_slovene
    /// [`EnglishWordMeaningModel`]: crate::entities::EnglishWordMeaningModel
    /// [`SloveneWordMeaningModel`]: crate::entities::SloveneWordMeaningModel
    pub struct WordMeaningModel {
        pub word_id: WordId,

        pub word_meaning_id: WordMeaningId,

        pub created_at: DateTime<Utc>,

        pub last_modified_at: DateTime<Utc>,
    }
}

pub use external::*;



impl IntoExternalModel for internal::InternalWordMeaningModel {
    type ExternalModel = WordMeaningModel;

    fn into_external_model(self) -> Self::ExternalModel {
        let meaning_id = WordMeaningId::new(self.id);
        let word_id = WordId::new(self.word_id);

        Self::ExternalModel {
            word_meaning_id: meaning_id,
            word_id,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
        }
    }
}
