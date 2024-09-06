use chrono::{DateTime, Utc};
use kolomoni_core::id::EnglishWordMeaningId;
use uuid::Uuid;

use crate::IntoModel;




pub struct Model {
    pub id: EnglishWordMeaningId,

    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,
}



pub(super) struct IntermediateModel {
    pub(super) word_meaning_id: Uuid,

    pub(super) disambiguation: Option<String>,

    pub(super) abbreviation: Option<String>,

    pub(super) description: Option<String>,

    pub(super) created_at: DateTime<Utc>,

    pub(super) last_modified_at: DateTime<Utc>,
}

impl IntoModel for IntermediateModel {
    type Model = Model;

    fn into_model(self) -> Self::Model {
        let id = EnglishWordMeaningId::new(self.word_meaning_id);

        Self::Model {
            id,
            disambiguation: self.disambiguation,
            abbreviation: self.abbreviation,
            description: self.description,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
        }
    }
}
