use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{
    EnglishWord,
    EnglishWordMeaningWithDetails,
    SloveneWord,
    SloveneWordMeaningWithDetails,
};



#[derive(Deserialize, Clone, PartialEq, Eq, ToSchema)]
#[cfg_attr(feature = "serde_impls_for_client_on_models", derive(serde::Serialize))]
#[schema(
    example = json!({
        "search_query": "hit points"
    })
)]
pub struct SearchRequest {
    /// Search query.
    pub search_query: String,
}




#[derive(Debug, Serialize, ToSchema)]
#[serde(tag = "type")]
#[cfg_attr(
    feature = "serde_impls_for_client_on_models",
    derive(serde::Deserialize)
)]
pub enum SearchedWordMeaning {
    #[serde(rename = "english")]
    English {
        result_score: f32,
        word: EnglishWord,
        word_meaning: EnglishWordMeaningWithDetails,
    },

    #[serde(rename = "slovene")]
    Slovene {
        result_score: f32,
        word: SloveneWord,
        word_meaning: SloveneWordMeaningWithDetails,
    },
}



#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(
    feature = "serde_impls_for_client_on_models",
    derive(serde::Deserialize)
)]
pub struct SearchResponse {
    pub word_meanings: Vec<SearchedWordMeaning>,
}
