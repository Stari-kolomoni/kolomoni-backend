use uuid::Uuid;

use crate::{entities::word::WordId, IntoModel};


#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WordMeaningId(Uuid);

impl WordMeaningId {
    #[inline]
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    #[inline]
    pub fn into_inner(self) -> Uuid {
        self.0
    }
}



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
