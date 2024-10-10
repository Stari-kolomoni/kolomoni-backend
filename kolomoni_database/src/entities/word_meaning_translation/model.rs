use chrono::{DateTime, Utc};
use kolomoni_core::ids::{EnglishWordMeaningId, SloveneWordMeaningId, UserId};
use uuid::Uuid;

use crate::IntoExternalModel;



pub struct WordMeaningTranslationModel {
    pub slovene_word_meaning_id: SloveneWordMeaningId,

    pub english_word_meaning_id: EnglishWordMeaningId,

    pub translated_at: DateTime<Utc>,

    pub translated_by: Option<UserId>,
}



pub struct InternalWordMeaningTranslationModel {
    pub(crate) slovene_word_meaning_id: Uuid,

    pub(crate) english_word_meaning_id: Uuid,

    pub(crate) translated_at: DateTime<Utc>,

    pub(crate) translated_by: Option<Uuid>,
}

impl IntoExternalModel for InternalWordMeaningTranslationModel {
    type ExternalModel = WordMeaningTranslationModel;

    fn into_external_model(self) -> Self::ExternalModel {
        let slovene_word_meaning_id = SloveneWordMeaningId::new(self.slovene_word_meaning_id);
        let english_word_meaning_id = EnglishWordMeaningId::new(self.english_word_meaning_id);

        let translated_by = self.translated_by.map(UserId::new);


        Self::ExternalModel {
            slovene_word_meaning_id,
            english_word_meaning_id,
            translated_at: self.translated_at,
            translated_by,
        }
    }
}
