use chrono::{DateTime, Utc};
use kolomoni_core::ids::CategoryId;
use uuid::Uuid;

use crate::IntoExternalModel;


pub struct CategoryModel {
    pub id: CategoryId,

    pub parent_category_id: Option<CategoryId>,

    pub slovene_name: String,

    pub english_name: String,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,
}


pub struct InternalCategoryModel {
    pub(crate) id: Uuid,

    pub(crate) parent_category_id: Option<Uuid>,

    pub(crate) name_sl: String,

    pub(crate) name_en: String,

    pub(crate) created_at: DateTime<Utc>,

    pub(crate) last_modified_at: DateTime<Utc>,
}

impl IntoExternalModel for InternalCategoryModel {
    type ExternalModel = CategoryModel;

    fn into_external_model(self) -> Self::ExternalModel {
        let id = CategoryId::new(self.id);
        let parent_category_id = self.parent_category_id.map(CategoryId::new);

        Self::ExternalModel {
            id,
            parent_category_id,
            slovene_name: self.name_sl,
            english_name: self.name_en,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
        }
    }
}
