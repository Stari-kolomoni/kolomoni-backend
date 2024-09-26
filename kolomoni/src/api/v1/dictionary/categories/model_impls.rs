use kolomoni_core::api_models::{
    CategoriesResponse,
    Category,
    CategoryCreationResponse,
    CategoryResponse,
};
use kolomoni_database::entities;

use crate::{api::traits::IntoApiModel, impl_json_response_builder};



impl_json_response_builder!(CategoryCreationResponse);


impl IntoApiModel<Category> for entities::CategoryModel {
    fn into_api_model(self) -> Category {
        Category {
            id: self.id,
            english_name: self.english_name,
            slovene_name: self.slovene_name,
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
        }
    }
}


impl_json_response_builder!(CategoriesResponse);
impl_json_response_builder!(CategoryResponse);
