use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::SloveneWordMeaningWithCategoriesAndTranslations;
use crate::id::SloveneWordId;



#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
#[schema(
    example = json!({
        "id": "018dbe00-266e-7398-abd2-0906df0aa345",
        "lemma": "pustolovec",
        "disambiguation": "lik",
        "description": "Igrani ali neigrani liki, ki se odpravijo na pustolovščino.",
        "created_at": "2023-06-27T20:34:27.217273Z",
        "last_modified_at": "2023-06-27T20:34:27.217273Z"
    })
)]
pub struct SloveneWordWithMeanings {
    /// Internal UUID of the word.
    pub id: SloveneWordId,

    /// An abstract or base form of the word.
    pub lemma: String,

    /// When the word was created.
    pub created_at: DateTime<Utc>,

    /// When the word was last modified.
    ///
    /// TODO In the future, this might include last modification time
    ///      of the linked suggestion and translation relationships.
    pub last_modified_at: DateTime<Utc>,

    pub meanings: Vec<SloveneWordMeaningWithCategoriesAndTranslations>,
}




#[derive(Serialize, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "serde_impls_for_client_on_models", derive(Deserialize))]
pub struct SloveneWordsResponse {
    pub slovene_words: Vec<SloveneWordWithMeanings>,
}


#[derive(Deserialize, Clone, PartialEq, Eq, Debug, ToSchema, Default)]
#[cfg_attr(feature = "serde_impls_for_client_on_models", derive(Serialize))]
pub struct SloveneWordFilters {
    pub last_modified_after: Option<DateTime<Utc>>,
}


#[derive(Deserialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "serde_impls_for_client_on_models", derive(Serialize))]
pub struct SloveneWordsListRequest {
    pub filters: Option<SloveneWordFilters>,
}



#[derive(Deserialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "serde_impls_for_client_on_models", derive(Serialize))]
#[schema(
    example = json!({
        "lemma": "pustolovec"
    })
)]
pub struct SloveneWordCreationRequest {
    pub lemma: String,
}


#[derive(Serialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "serde_impls_for_client_on_models", derive(Deserialize))]
#[schema(
    example = json!({
        "word": {
            "id": "018dbe00-266e-7398-abd2-0906df0aa345",
            "lemma": "pustolovec",
            "disambiguation": "lik",
            "description": "Igrani ali neigrani liki, ki se odpravijo na pustolovščino.",
            "added_at": "2023-06-27T20:34:27.217273Z",
            "last_edited_at": "2023-06-27T20:34:27.217273Z"
        }
    })
)]
pub struct SloveneWordCreationResponse {
    pub word: SloveneWordWithMeanings,
}

#[derive(Serialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "serde_impls_for_client_on_models", derive(Deserialize))]
pub struct SloveneWordInfoResponse {
    pub word: SloveneWordWithMeanings,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug, ToSchema, Default)]
pub struct SloveneWordUpdateRequest {
    pub lemma: Option<String>,
}
