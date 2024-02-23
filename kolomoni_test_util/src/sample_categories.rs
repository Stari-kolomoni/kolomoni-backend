use http::{Method, StatusCode};
use kolomoni::api::v1::dictionary::{
    categories::{CategoryCreationRequest, CategoryCreationResponse},
    Category,
};

use crate::TestServer;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SampleCategory {
    Lik,
    Vescina,
    Razred,
    DejavnostiInSpopad,
}


impl SampleCategory {
    pub fn english_name(&self) -> &'static str {
        match self {
            SampleCategory::Lik => "character",
            SampleCategory::Vescina => "skill",
            SampleCategory::Razred => "class",
            SampleCategory::DejavnostiInSpopad => "activities and combat",
        }
    }

    pub fn slovene_name(&self) -> &'static str {
        match self {
            SampleCategory::Lik => "lik",
            SampleCategory::Vescina => "veščina",
            SampleCategory::Razred => "razred",
            SampleCategory::DejavnostiInSpopad => "dejavnosti in spopad",
        }
    }
}


pub async fn create_sample_category(
    server: &TestServer,
    access_token: &str,
    category: SampleCategory,
) -> Category {
    let creation_response = server
        .request(Method::POST, "/api/v1/dictionary/category")
        .with_access_token(access_token)
        .with_json_body(CategoryCreationRequest {
            slovene_name: category.slovene_name().to_string(),
            english_name: category.english_name().to_string(),
        })
        .send()
        .await;

    creation_response.assert_status_equals(StatusCode::OK);

    creation_response
        .json_body::<CategoryCreationResponse>()
        .category
}
