use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{entities::word::WordId, IntoModel};

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SloveneWordId(Uuid);

impl SloveneWordId {
    #[inline]
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    #[inline]
    pub fn into_word_id(self) -> WordId {
        WordId::new(self.0)
    }

    #[inline]
    pub fn into_inner(self) -> Uuid {
        self.0
    }
}


pub struct ExtendedModel {
    pub word_id: SloveneWordId,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,

    pub lemma: String,
}


pub struct Model {
    pub word_id: SloveneWordId,

    pub lemma: String,
}



pub(super) struct IntermediateExtendedModel {
    pub(super) word_id: Uuid,

    pub(super) lemma: String,

    pub(super) created_at: DateTime<Utc>,

    pub(super) last_modified_at: DateTime<Utc>,
}

impl IntoModel for IntermediateExtendedModel {
    type Model = ExtendedModel;

    fn into_model(self) -> Self::Model {
        let word_id = SloveneWordId::new(self.word_id);

        Self::Model {
            word_id,
            lemma: self.lemma,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
        }
    }
}
