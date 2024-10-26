use kolomoni_core::ids::{CategoryId, WordMeaningId};
use uuid::Uuid;

use crate::IntoExternalModel;


pub struct WordMeaningCategory {
    pub word_meaning_id: WordMeaningId,

    pub category_id: CategoryId,
}


pub struct InternalWordMeaningCategory {
    pub(crate) word_meaning_id: Uuid,

    pub(crate) category_id: Uuid,
}

impl IntoExternalModel for InternalWordMeaningCategory {
    type ExternalModel = WordMeaningCategory;

    fn into_external_model(self) -> Self::ExternalModel {
        WordMeaningCategory {
            word_meaning_id: WordMeaningId::new(self.word_meaning_id),
            category_id: CategoryId::new(self.category_id),
        }
    }
}
