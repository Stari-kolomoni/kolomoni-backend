use std::borrow::Cow;

use kolomoni_core::ids::WordId;

use super::WordLanguage;
use crate::TryIntoExternalModel;


pub(crate) mod internal {
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    /// An internal language-agnostic word model, as queried from the database.
    /// Contains the fields that all word models share, which are currently:
    /// UUID and the creation and modification timestamp.
    ///
    /// For language-specific internal models, see the [`word_english`] and [`word_slovene`] modules
    /// (e.g. [`InternalEnglishWordModel`], [`InternalSloveneWordModel`], etc.).
    ///
    ///
    /// [`word_english`]: crate::entities::word_english
    /// [`word_slovene`]: crate::entities::word_slovene
    /// [`InternalEnglishWordModel`]: crate::entities::InternalEnglishWordModel
    /// [`InternalSloveneWordModel`]: crate::entities::InternalSloveneWordModel
    pub(crate) struct InternalWordModel {
        pub(crate) id: Uuid,

        pub(crate) language_code: String,

        pub(crate) created_at: DateTime<Utc>,

        pub(crate) last_modified_at: DateTime<Utc>,
    }
}


mod external {
    use chrono::{DateTime, Utc};
    use kolomoni_core::ids::WordId;

    use crate::entities::word::WordLanguage;

    /// An external language-agnostic word model. Contains the fields that all word models
    /// share, which are currently a UUID and the creation and modification timestamps.
    ///
    /// For language-specific external models, see the [`word_english`] and [`word_slovene`] modules
    /// (e.g. [`EnglishWordModel`], [`SloveneWordModel`], etc.).
    ///
    ///
    /// # Note
    /// The phrasing "external" in this context refers to a public API that this crate exposes (`pub`-visible models).
    /// Notably, this phrasing *does not* imply that this type is exposed in the sense of the REST API
    /// that the Stari Kolomoni backend server exposes. For those REST API models, see the [`kolomoni_core`] crate.
    ///
    ///
    /// [`word_english`]: crate::entities::word_english
    /// [`word_slovene`]: crate::entities::word_slovene
    /// [`EnglishWordModel`]: crate::entities::EnglishWordModel
    /// [`SloveneWordModel`]: crate::entities::SloveneWordModel
    pub struct WordModel {
        pub id: WordId,

        pub language: WordLanguage,

        pub created_at: DateTime<Utc>,

        pub last_modified_at: DateTime<Utc>,
    }
}

pub use external::*;




impl TryIntoExternalModel for internal::InternalWordModel {
    type ExternalModel = WordModel;
    type Error = Cow<'static, str>;

    fn try_into_external_model(self) -> Result<Self::ExternalModel, Self::Error> {
        let language =
            WordLanguage::from_ietf_bcp_47_language_tag(&self.language_code).ok_or_else(|| {
                Cow::from(format!(
                    "unexpected language tag: \"{}\" (expected \"en\" or \"sl\")",
                    self.language_code
                ))
            })?;

        Ok(Self::ExternalModel {
            id: WordId::new(self.id),
            language,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
        })
    }
}
