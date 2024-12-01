use super::{InternalEnglishWordId, ToOutputModel};
use crate::analyzer::{clean_up_str, insert_only_set::InternalId};


pub struct IntermediateEnglishWord {
    internal_id: InternalEnglishWordId,

    lemma: String,
}

impl IntermediateEnglishWord {
    pub fn from_raw_data(raw_lemma: String) -> Self {
        Self {
            internal_id: InternalEnglishWordId::generate(),
            lemma: clean_up_str(&raw_lemma).to_string(),
        }
    }
}

impl InternalId for IntermediateEnglishWord {
    type InternalId = InternalEnglishWordId;

    fn internal_id(&self) -> Self::InternalId {
        self.internal_id
    }
}



#[derive(Debug, Clone)]
pub struct EnglishWord {
    internal_id: InternalEnglishWordId,

    pub lemma: String,
}

impl InternalId for EnglishWord {
    type InternalId = InternalEnglishWordId;

    fn internal_id(&self) -> Self::InternalId {
        self.internal_id
    }
}



impl ToOutputModel for IntermediateEnglishWord {
    type OutputModel = EnglishWord;

    fn to_output_model(&self) -> Self::OutputModel {
        Self::OutputModel {
            internal_id: self.internal_id,
            lemma: self.lemma.clone(),
        }
    }
}
