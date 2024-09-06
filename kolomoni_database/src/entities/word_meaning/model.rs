use kolomoni_core::id::{WordId, WordMeaningId};
use uuid::Uuid;

use crate::IntoModel;



pub struct Model {
    pub meaning_id: WordMeaningId,

    pub word_id: WordId,
}


pub(super) struct IntermediateModel {
    pub(super) id: Uuid,

    pub(super) word_id: Uuid,
}

impl IntoModel for IntermediateModel {
    type Model = Model;

    fn into_model(self) -> Self::Model {
        let meaning_id = WordMeaningId::new(self.id);
        let word_id = WordId::new(self.word_id);

        Self::Model {
            meaning_id,
            word_id,
        }
    }
}
