use std::borrow::Cow;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::TryIntoModel;



#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WordId(pub(crate) Uuid);

impl WordId {
    #[inline]
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    #[inline]
    pub fn into_inner(self) -> Uuid {
        self.0
    }
}



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



pub struct Model {
    pub id: WordId,

    pub language: WordLanguage,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,
}


pub(super) struct IntermediateModel {
    pub(super) id: Uuid,

    pub(super) language: String,

    pub(super) created_at: DateTime<Utc>,

    pub(super) last_modified_at: DateTime<Utc>,
}

impl TryIntoModel for IntermediateModel {
    type Model = Model;
    type Error = Cow<'static, str>;

    fn try_into_model(self) -> Result<Self::Model, Self::Error> {
        let language =
            WordLanguage::from_ietf_bcp_47_language_tag(&self.language).ok_or_else(|| {
                Cow::from(format!(
                    "unexpected language tag \"{}\", expected \"en\" or \"sl\"",
                    self.language
                ))
            })?;

        Ok(Self::Model {
            id: WordId::new(self.id),
            language,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
        })
    }
}
