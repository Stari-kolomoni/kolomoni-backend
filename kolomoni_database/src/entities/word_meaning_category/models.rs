use kolomoni_core::ids::{CategoryId, WordMeaningId};

use crate::IntoExternalModel;


pub(crate) mod internal {
    use uuid::Uuid;

    pub struct InternalWordMeaningCategory {
        pub(crate) word_meaning_id: Uuid,

        pub(crate) category_id: Uuid,
    }
}


mod external {
    use kolomoni_core::ids::{CategoryId, WordMeaningId};

    pub struct WordMeaningCategory {
        pub word_meaning_id: WordMeaningId,

        pub category_id: CategoryId,
    }
}

pub use external::*;



impl IntoExternalModel for internal::InternalWordMeaningCategory {
    type ExternalModel = WordMeaningCategory;

    fn into_external_model(self) -> Self::ExternalModel {
        WordMeaningCategory {
            word_meaning_id: WordMeaningId::new(self.word_meaning_id),
            category_id: CategoryId::new(self.category_id),
        }
    }
}
