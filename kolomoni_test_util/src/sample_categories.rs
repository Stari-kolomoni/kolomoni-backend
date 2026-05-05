use chrono::Utc;
use kolomoni_api_client::{
    api::dictionary::categories::{CategoryToCreate, DictionaryCategoriesAuthenticatedEndpoints},
    client::AuthenticatedKolomoniClient,
};
use kolomoni_core::api_models::Category;

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

    pub async fn create(&self, client: &AuthenticatedKolomoniClient) -> Category {
        let before_category_creation = Utc::now();

        let new_category = client
            .categories()
            .create_category(CategoryToCreate {
                english_category_name: self.english_name().to_owned(),
                slovene_category_name: self.slovene_name().to_owned(),
                parent_category_id: None,
            })
            .send()
            .await
            .expect("failed to create new sample category");

        let after_category_creation = Utc::now();


        assert_eq!(new_category.english_name, self.english_name());
        assert_eq!(new_category.slovene_name, self.slovene_name());
        assert!(new_category.parent_category_id.is_none());

        assert!(new_category.created_at >= before_category_creation);
        assert!(new_category.created_at <= after_category_creation);

        assert_eq!(
            new_category.created_at,
            new_category.last_modified_at
        );


        new_category
    }
}
