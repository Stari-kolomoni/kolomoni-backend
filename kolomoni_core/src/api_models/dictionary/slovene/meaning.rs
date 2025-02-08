use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    api_models::{EnglishWord, EnglishWordMeaningWithShallowDetails},
    ids::{CategoryId, SloveneWordMeaningId, UserId},
};



#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct SloveneWordMeaning {
    #[schema(value_type = uuid::Uuid)]
    pub word_meaning_id: SloveneWordMeaningId,

    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,
}


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct SloveneWordMeaningWithShallowDetails {
    #[schema(value_type = uuid::Uuid)]
    pub word_meaning_id: SloveneWordMeaningId,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,

    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,

    #[schema(value_type = Vec<uuid::Uuid>)]
    pub categories: Vec<CategoryId>,
}


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct SloveneWordMeaningWithDetails {
    #[schema(value_type = uuid::Uuid)]
    pub word_meaning_id: SloveneWordMeaningId,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,

    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,

    #[schema(value_type = Vec<uuid::Uuid>)]
    pub categories: Vec<CategoryId>,

    pub translations: Vec<EnglishTranslation>,
}


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct EnglishTranslation {
    pub word: EnglishWord,

    pub word_meaning: EnglishWordMeaningWithShallowDetails,

    pub translated_at: DateTime<Utc>,

    #[schema(value_type = Option<uuid::Uuid>)]
    pub translated_by: Option<UserId>,
}




#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct SloveneWordMeaningsResponse {
    pub meanings: Vec<SloveneWordMeaningWithDetails>,
}


// TODO could be nice to submit initial categories with this as well? (see also english version of this)
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct NewSloveneWordMeaningRequest {
    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,
}


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct NewSloveneWordMeaningCreatedResponse {
    pub meaning: SloveneWordMeaning,
}


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct SloveneWordMeaningUpdateRequest {
    #[serde(default, with = "::serde_with::rust::double_option")]
    pub disambiguation: Option<Option<String>>,

    #[serde(default, with = "::serde_with::rust::double_option")]
    pub abbreviation: Option<Option<String>>,

    #[serde(default, with = "::serde_with::rust::double_option")]
    pub description: Option<Option<String>>,
}


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct SloveneWordMeaningUpdatedResponse {
    pub meaning: SloveneWordMeaningWithDetails,
}
