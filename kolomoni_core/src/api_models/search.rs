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
#[cfg_attr(
    feature = "serde_impls_for_client_on_models",
    derive(serde::Deserialize)
)]
pub struct ScoredEnglishWordMeaningWithDetails {
    pub score: f32,

    #[serde(flatten)]
    pub english_word_meaning: EnglishWordMeaningWithDetails,
}


#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(
    feature = "serde_impls_for_client_on_models",
    derive(serde::Deserialize)
)]
pub struct ScoredSloveneWordMeaningWithDetails {
    pub score: f32,

    #[serde(flatten)]
    pub slovene_word_meaning: SloveneWordMeaningWithDetails,
}


#[derive(Debug, Serialize, ToSchema)]
#[serde(tag = "type")]
#[cfg_attr(
    feature = "serde_impls_for_client_on_models",
    derive(serde::Deserialize)
)]
pub enum WordSearchResult {
    #[serde(rename = "english")]
    English {
        /// Cumulative word match score considering all the matched
        /// word meanings of this word.
        cumulative_score: f32,

        word: EnglishWord,

        /// Note that this array won't always contain all the meanings
        /// associated with the given word, because some meanings might
        /// just not match for a given search term.
        word_meanings: Vec<ScoredEnglishWordMeaningWithDetails>,
    },

    #[serde(rename = "slovene")]
    Slovene {
        /// Cumulative word match score considering all the matched
        /// word meanings of this word.
        cumulative_score: f32,

        word: SloveneWord,

        /// Note that this array won't always contain all the meanings
        /// associated with the given word, because some meanings might
        /// just not match for a given search term.
        word_meanings: Vec<ScoredSloveneWordMeaningWithDetails>,
    },
}

impl WordSearchResult {
    pub fn cumulative_score(&self) -> f32 {
        match self {
            Self::English {
                cumulative_score, ..
            } => *cumulative_score,
            Self::Slovene {
                cumulative_score, ..
            } => *cumulative_score,
        }
    }
}



#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(
    feature = "serde_impls_for_client_on_models",
    derive(serde::Deserialize)
)]
pub struct SearchResponse {
    pub search_results: Vec<WordSearchResult>,
}
