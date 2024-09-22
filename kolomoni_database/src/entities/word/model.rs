use std::borrow::Cow;

use chrono::{DateTime, Utc};
use kolomoni_core::id::WordId;
use uuid::Uuid;

use crate::TryIntoExternalModel;


#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum WordLanguage {
    Slovene,
    English,
}

impl WordLanguage {
    pub fn from_ietf_bcp_47_language_tag(language_tag: &str) -> Option<Self> {
        match language_tag {
            "sl" => Some(Self::Slovene),
            "en" => Some(Self::English),
            _ => None,
        }
    }

    pub fn to_ietf_bcp_47_language_tag(self) -> &'static str {
        match self {
            WordLanguage::Slovene => "sl",
            WordLanguage::English => "en",
        }
    }
}



pub struct WordModel {
    pub id: WordId,

    pub language: WordLanguage,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,
}


pub struct InternalWordModel {
    pub(crate) id: Uuid,

    pub(crate) language_code: String,

    pub(crate) created_at: DateTime<Utc>,

    pub(crate) last_modified_at: DateTime<Utc>,
}

impl TryIntoExternalModel for InternalWordModel {
    type ExternalModel = WordModel;
    type Error = Cow<'static, str>;

    fn try_into_external_model(self) -> Result<Self::ExternalModel, Self::Error> {
        let language =
            WordLanguage::from_ietf_bcp_47_language_tag(&self.language_code).ok_or_else(|| {
                Cow::from(format!(
                    "unexpected language tag \"{}\", expected \"en\" or \"sl\"",
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
