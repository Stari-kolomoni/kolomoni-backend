use kolomoni_core::{api_models::CategoryCreationRequest, ids::CategoryId};
use reqwest::StatusCode;

use crate::{request::RequestBuilder, Client, ClientResult};



pub struct CategoryToCreate {
    pub parent_category_id: Option<CategoryId>,

    pub slovene_category_name: String,

    pub english_category_name: String,
}


pub struct DictionaryCategoriesApi<'c> {
    client: &'c Client,
}

impl<'c> DictionaryCategoriesApi<'c> {
    pub async fn get_all_categories(&self) -> ClientResult<()> {
        let response = RequestBuilder::get(self.client)
            .endpoint_url("/dictionary/category")
            .send()
            .await?;

        todo!();
    }

    pub async fn create_category(&self, category: CategoryToCreate) -> ClientResult<()> {
        let response = RequestBuilder::post(self.client)
            .endpoint_url("/dictionary/category")
            .json(&CategoryCreationRequest {
                parent_category_id: category.parent_category_id.map(|id| id.into_uuid()),
                english_name: category.english_category_name,
                slovene_name: category.slovene_category_name,
            })
            .send()
            .await?;


        let response_status = response.status();

        // TODO continue from here
        if response_status == StatusCode::OK {
            todo!();
        } else if response_status == StatusCode::CONFLICT {
            todo!();
        } else if response_status == StatusCode::FORBIDDEN {
            todo!();
        } else {
            todo!();
        }



        todo!("use the new request builder type");
    }
}
