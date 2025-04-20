use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    api_models::{SloveneWord, SloveneWordMeaning},
    ids::{CategoryId, EnglishWordMeaningId, UserId},
};


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct EnglishWordMeaning {
    #[schema(value_type = uuid::Uuid)]
    pub word_meaning_id: EnglishWordMeaningId,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,

    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,
}


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct EnglishWordMeaningWithShallowDetails {
    #[schema(value_type = uuid::Uuid)]
    pub word_meaning_id: EnglishWordMeaningId,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,

    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,

    #[schema(value_type = Vec<uuid::Uuid>)]
    pub categories: Vec<CategoryId>,
}


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct EnglishWordMeaningWithDetails {
    #[schema(value_type = uuid::Uuid)]
    pub word_meaning_id: EnglishWordMeaningId,

    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,

    #[schema(value_type = Vec<uuid::Uuid>)]
    pub categories: Vec<CategoryId>,

    pub translations: Vec<SloveneTranslation>,
}


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct SloveneTranslation {
    pub word: SloveneWord,

    pub word_meaning: SloveneWordMeaning,

    pub translated_at: DateTime<Utc>,

    #[schema(value_type = Option<uuid::Uuid>)]
    pub translated_by: Option<UserId>,
}




#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct EnglishWordMeaningsResponse {
    pub meanings: Vec<EnglishWordMeaningWithDetails>,
}


// TODO could be nice to submit initial categories with this as well?
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct NewEnglishWordMeaningRequest {
    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,
}


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct NewEnglishWordMeaningCreatedResponse {
    pub meaning: EnglishWordMeaning,
}


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct EnglishWordMeaningUpdateRequest {
    #[serde(default, with = "::serde_with::rust::double_option")]
    pub disambiguation: Option<Option<String>>,

    #[serde(default, with = "::serde_with::rust::double_option")]
    pub abbreviation: Option<Option<String>>,

    #[serde(default, with = "::serde_with::rust::double_option")]
    pub description: Option<Option<String>>,
}


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct EnglishWordMeaningUpdatedResponse {
    pub meaning: EnglishWordMeaningWithDetails,
}
