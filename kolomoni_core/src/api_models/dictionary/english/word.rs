use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use super::EnglishWordMeaningWithCategoriesAndTranslations;
use crate::id::EnglishWordId;

// TODO needs updated example
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
#[schema(
    example = json!({
        "id": "018dbe00-266e-7398-abd2-0906df0aa345",
        "lemma": "adventurer",
        "disambiguation": "character",
        "description": "Playable or non-playable character.",
        "created_at": "2023-06-27T20:34:27.217273Z",
        "last_modified_at": "2023-06-27T20:34:27.217273Z",
        "suggested_translations": [],
        "translations": [
            {
                "id": "018dbe00-266e-7398-abd2-0906df0aa346",
                "lemma": "pustolovec",
                "disambiguation": "lik",
                "description": "Igrani ali neigrani liki, ki se odpravijo na pustolovščino.",
                "created_at": "2023-06-27T20:34:27.217273Z",
                "last_modified_at": "2023-06-27T20:34:27.217273Z"
            }
        ]
    })
)]
pub struct EnglishWordWithMeanings {
    /// Word UUID.
    pub id: EnglishWordId,

    /// An abstract or base form of the word.
    pub lemma: String,

    /// When the word was created.
    pub created_at: DateTime<Utc>,

    /// When the word was last modified.
    /// This includes the last creation or deletion time of the
    /// suggestion or translation linked to this word.
    pub last_modified_at: DateTime<Utc>,

    pub meanings: Vec<EnglishWordMeaningWithCategoriesAndTranslations>,
}


#[derive(Serialize, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "serde_impls_for_client_on_models", derive(Deserialize))]
pub struct EnglishWordsResponse {
    pub english_words: Vec<EnglishWordWithMeanings>,
}



#[derive(Deserialize, Clone, PartialEq, Eq, Debug, ToSchema, IntoParams)]
#[cfg_attr(feature = "serde_impls_for_client_on_models", derive(Serialize))]
pub struct EnglishWordsListRequest {
    pub last_modified_after: Option<DateTime<Utc>>,
}


#[derive(Deserialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "serde_impls_for_client_on_models", derive(Serialize))]
#[schema(
    example = json!({
        "lemma": "adventurer"
    })
)]
pub struct EnglishWordCreationRequest {
    pub lemma: String,
}


#[derive(Serialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "serde_impls_for_client_on_models", derive(Deserialize))]
#[schema(
    example = json!({
        "word": {
            "id": "018dbe00-266e-7398-abd2-0906df0aa345",
            "lemma": "adventurer",
            "added_at": "2023-06-27T20:34:27.217273Z",
            "last_edited_at": "2023-06-27T20:34:27.217273Z"
        }
    })
)]
pub struct EnglishWordCreationResponse {
    pub word: EnglishWordWithMeanings,
}


#[derive(Serialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "serde_impls_for_client_on_models", derive(Deserialize))]
pub struct EnglishWordInfoResponse {
    pub word: EnglishWordWithMeanings,
}


#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug, ToSchema, Default)]
pub struct EnglishWordUpdateRequest {
    pub lemma: Option<String>,
}
