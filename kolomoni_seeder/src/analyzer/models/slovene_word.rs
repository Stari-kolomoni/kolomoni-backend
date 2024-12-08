use super::{InternalSloveneWordId, ToOutputModel};
use crate::analyzer::{clean_up_string, insert_only_set::InternalId};

pub struct IntermediateSloveneWord {
    internal_id: InternalSloveneWordId,

    lemma: String,
}

impl IntermediateSloveneWord {
    pub fn from_raw_data(raw_lemma: String) -> Self {
        Self {
            internal_id: InternalSloveneWordId::generate(),
            lemma: clean_up_string(raw_lemma),
        }
    }
}

impl InternalId for IntermediateSloveneWord {
    type InternalId = InternalSloveneWordId;

    fn internal_id(&self) -> Self::InternalId {
        self.internal_id
    }
}


#[derive(Debug, Clone)]
pub struct SloveneWord {
    internal_id: InternalSloveneWordId,

    pub lemma: String,
}

impl InternalId for SloveneWord {
    type InternalId = InternalSloveneWordId;

    fn internal_id(&self) -> Self::InternalId {
        self.internal_id
    }
}


impl ToOutputModel for IntermediateSloveneWord {
    type OutputModel = SloveneWord;

    fn to_output_model(&self) -> Self::OutputModel {
        Self::OutputModel {
            internal_id: self.internal_id,
            lemma: self.lemma.clone(),
        }
    }
}
