use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::ids::CategoryId;


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct Category {
    pub id: CategoryId,

    pub slovene_name: String,

    pub english_name: String,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,
}



#[derive(Deserialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "serde_impls_for_client_on_models", derive(Serialize))]
#[schema(
    example = json!({
        "slovene_name": "Dejavnosti in spopad",
        "english_name": "Activities and Combat",
    })
)]
pub struct CategoryCreationRequest {
    pub parent_category_id: Option<Uuid>,
    pub slovene_name: String,
    pub english_name: String,
}



#[derive(Serialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "serde_impls_for_client_on_models", derive(Deserialize))]
#[schema(
    example = json!({
        "category": {
            "id": 1,
            "slovene_name": "Dejavnosti in spopad",
            "english_name": "Activities and Combat",
            "created_at": "2023-06-27T20:34:27.217273Z",
            "last_modified_at": "2023-06-27T20:34:27.217273Z",
        }
    })
)]
pub struct CategoryCreationResponse {
    pub category: Category,
}



#[derive(Serialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "serde_impls_for_client_on_models", derive(Deserialize))]
pub struct CategoriesResponse {
    pub categories: Vec<Category>,
}



#[derive(Serialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "serde_impls_for_client_on_models", derive(Deserialize))]
#[schema(
    example = json!({
        "category": {
            "id": 1,
            "slovene_name": "Dejavnosti in spopad",
            "english_name": "Activities and Combat",
        }
    })
)]
pub struct CategoryResponse {
    pub category: Category,
}


#[derive(Deserialize, Clone, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "serde_impls_for_client_on_models", derive(Serialize))]
#[schema(
    example = json!({
        "slovene_name": "Dejavnosti in spopad",
        "english_name": "Activities and Combat",
    })
)]
pub struct CategoryUpdateRequest {
    /// # Interpreting the double option
    /// To distinguish from an unset and a null JSON value, this field is a
    /// double option. `None` indicates the field was not present
    /// (i.e. that the parent category should not change as part of this update),
    /// while `Some(None)` indicates it was set to `null`
    /// (i.e. that the parent category should be cleared).
    ///
    /// See also: [`serde_with::rust::double_option`].
    #[serde(default, with = "::serde_with::rust::double_option")]
    pub new_parent_category_id: Option<Option<Uuid>>,

    pub new_slovene_name: Option<String>,

    pub new_english_name: Option<String>,
}
