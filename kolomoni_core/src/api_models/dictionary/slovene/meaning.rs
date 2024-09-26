use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    api_models::ShallowEnglishWordMeaning,
    id::{CategoryId, SloveneWordMeaningId},
};


// TODO this should actually probably be named SloveneWordMeaningWithCategories
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct ShallowSloveneWordMeaning {
    pub meaning_id: SloveneWordMeaningId,

    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,

    pub categories: Vec<CategoryId>,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,
}


// TODO refactor these names, this one is the same as ShallowSloveneWordMeaning, but without categories
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct SloveneWordMeaning {
    pub meaning_id: SloveneWordMeaningId,

    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct SloveneWordMeaningWithCategoriesAndTranslations {
    pub meaning_id: SloveneWordMeaningId,

    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,

    pub categories: Vec<CategoryId>,

    pub translates_into: Vec<ShallowEnglishWordMeaning>,
}



#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct SloveneWordMeaningsResponse {
    pub meanings: Vec<SloveneWordMeaningWithCategoriesAndTranslations>,
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
    pub meaning: SloveneWordMeaningWithCategoriesAndTranslations,
}
