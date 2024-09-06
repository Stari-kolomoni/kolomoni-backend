use chrono::{DateTime, Utc};
use kolomoni_core::id::EnglishWordId;
use uuid::Uuid;

use crate::IntoModel;




pub struct ExtendedModel {
    pub word_id: EnglishWordId,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,

    pub lemma: String,
}


pub struct Model {
    pub word_id: EnglishWordId,

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
        let word_id = EnglishWordId::new(self.word_id);

        Self::Model {
            word_id,
            lemma: self.lemma,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
        }
    }
}
