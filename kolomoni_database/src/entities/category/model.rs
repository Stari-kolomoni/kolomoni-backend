use chrono::{DateTime, Utc};
use kolomoni_core::id::CategoryId;
use uuid::Uuid;

use crate::IntoModel;


pub struct Model {
    pub id: CategoryId,

    pub parent_category_id: Option<CategoryId>,

    pub slovene_name: String,

    pub english_name: String,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,
}


pub(super) struct IntermediateModel {
    pub(super) id: Uuid,

    pub(super) parent_category_id: Option<Uuid>,

    pub(super) name_sl: String,

    pub(super) name_en: String,

    pub(super) created_at: DateTime<Utc>,

    pub(super) last_modified_at: DateTime<Utc>,
}

impl IntoModel for IntermediateModel {
    type Model = Model;

    fn into_model(self) -> Self::Model {
        let id = CategoryId::new(self.id);
        let parent_category_id = self.parent_category_id.map(CategoryId::new);

        Self::Model {
            id,
            parent_category_id,
            slovene_name: self.name_sl,
            english_name: self.name_en,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
        }
    }
}
