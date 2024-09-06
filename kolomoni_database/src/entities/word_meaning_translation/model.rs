use chrono::{DateTime, Utc};
use kolomoni_core::id::{EnglishWordMeaningId, SloveneWordMeaningId, UserId};
use uuid::Uuid;

use crate::IntoModel;



pub struct Model {
    pub slovene_word_meaning_id: SloveneWordMeaningId,

    pub english_word_meaning_id: EnglishWordMeaningId,

    pub translated_at: DateTime<Utc>,

    pub translated_by: Option<UserId>,
}



pub(super) struct IntermediateModel {
    pub(super) slovene_word_meaning_id: Uuid,

    pub(super) english_word_meaning_id: Uuid,

    pub(super) translated_at: DateTime<Utc>,

    pub(super) translated_by: Option<Uuid>,
}

impl IntoModel for IntermediateModel {
    type Model = Model;

    fn into_model(self) -> Self::Model {
        let slovene_word_meaning_id = SloveneWordMeaningId::new(self.slovene_word_meaning_id);
        let english_word_meaning_id = EnglishWordMeaningId::new(self.english_word_meaning_id);

        let translated_by = self.translated_by.map(UserId::new);


        Self::Model {
            slovene_word_meaning_id,
            english_word_meaning_id,
            translated_at: self.translated_at,
            translated_by,
        }
    }
}
