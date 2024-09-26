use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    api_models::ShallowSloveneWordMeaning,
    id::{CategoryId, EnglishWordMeaningId},
};

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct ShallowEnglishWordMeaning {
    pub meaning_id: EnglishWordMeaningId,

    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,

    pub categories: Vec<CategoryId>,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct EnglishWordMeaning {
    pub meaning_id: EnglishWordMeaningId,

    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,
}


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct EnglishWordMeaningWithCategoriesAndTranslations {
    pub meaning_id: EnglishWordMeaningId,

    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,

    pub categories: Vec<CategoryId>,

    pub translates_into: Vec<ShallowSloveneWordMeaning>,
}


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct EnglishWordMeaningsResponse {
    pub meanings: Vec<EnglishWordMeaningWithCategoriesAndTranslations>,
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
    pub meaning: EnglishWordMeaningWithCategoriesAndTranslations,
}
