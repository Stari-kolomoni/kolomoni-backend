use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct TranslationCreationRequest {
    pub english_word_meaning_id: Uuid,
    pub slovene_word_meaning_id: Uuid,
}


#[derive(
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Clone,
    Debug,
    ToSchema,
    IntoParams
)]
pub struct TranslationDeletionRequest {
    pub english_word_meaning_id: Uuid,
    pub slovene_word_meaning_id: Uuid,
}
