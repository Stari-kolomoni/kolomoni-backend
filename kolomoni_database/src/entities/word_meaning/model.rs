use kolomoni_core::ids::{WordId, WordMeaningId};
use uuid::Uuid;

use crate::IntoExternalModel;



pub struct WordMeaningModel {
    pub meaning_id: WordMeaningId,

    pub word_id: WordId,
}


pub struct InternalWordMeaningModel {
    pub(crate) id: Uuid,

    pub(crate) word_id: Uuid,
}

impl IntoExternalModel for InternalWordMeaningModel {
    type ExternalModel = WordMeaningModel;

    fn into_external_model(self) -> Self::ExternalModel {
        let meaning_id = WordMeaningId::new(self.id);
        let word_id = WordId::new(self.word_id);

        Self::ExternalModel {
            meaning_id,
            word_id,
        }
    }
}
