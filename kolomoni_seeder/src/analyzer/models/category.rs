use thiserror::Error;

use super::{InternalCategoryId, TryToOutputModelWithContext};
use crate::{
    analyzer::{
        clean_up_optional_string,
        clean_up_string,
        insert_only_set::{GrowingKeyedSet, InternalId},
    },
    parser::SeedCategoryRow,
};



pub enum IntermediateCategoryParentState {
    NoParent,
    ByName { english_name: String },
}

pub struct IntermediateCategory {
    internal_id: InternalCategoryId,

    english_name: String,
    english_description: Option<String>,

    slovene_name: String,
    slovene_description: Option<String>,

    parent_category: IntermediateCategoryParentState,
}

impl IntermediateCategory {
    pub fn from_seed_category_row(row: SeedCategoryRow) -> Self {
        IntermediateCategory {
            internal_id: InternalCategoryId::generate(),
            english_name: clean_up_string(row.english_name),
            english_description: clean_up_optional_string(row.english_description),
            slovene_name: clean_up_string(row.slovene_name),
            slovene_description: clean_up_optional_string(row.slovene_description),
            parent_category: match row.parent_category_english_name {
                Some(parent_category_name) => IntermediateCategoryParentState::ByName {
                    english_name: parent_category_name,
                },
                None => IntermediateCategoryParentState::NoParent,
            },
        }
    }

    pub fn english_name(&self) -> &str {
        &self.english_name
    }
}

impl InternalId for IntermediateCategory {
    type InternalId = InternalCategoryId;

    fn internal_id(&self) -> Self::InternalId {
        self.internal_id
    }
}


pub struct IntermediateCategoryOutputContext<'a> {
    intemediate_categories: &'a GrowingKeyedSet<IntermediateCategory>,
}

impl<'a> IntermediateCategoryOutputContext<'a> {
    pub fn new(intemediate_categories: &'a GrowingKeyedSet<IntermediateCategory>) -> Self {
        Self {
            intemediate_categories,
        }
    }

    fn category_by_english_name(
        &self,
        english_category_name: &str,
    ) -> Option<&IntermediateCategory> {
        let mut target_category_internal_id = None;

        for category in self.intemediate_categories.values() {
            if english_category_name == category.english_name {
                target_category_internal_id = Some(category.internal_id);
            }
        }

        let target_category_internal_id = target_category_internal_id?;

        self.intemediate_categories
            .get(&target_category_internal_id)
    }
}


#[derive(Debug, Error)]
pub enum IntermediateCategoryOutputError {
    #[error("no such parent category with ID: {}", .missing_parent_category_english_name)]
    ParentCategoryNotFoundByName {
        missing_parent_category_english_name: String,
    },
}


impl TryToOutputModelWithContext for IntermediateCategory {
    type Context<'c> = IntermediateCategoryOutputContext<'c>;
    type OutputModel = Category;
    type Error = IntermediateCategoryOutputError;

    fn try_to_output_model<'a>(
        &self,
        context: &'a Self::Context<'a>,
    ) -> Result<Self::OutputModel, Self::Error> {
        let IntermediateCategoryParentState::ByName {
            english_name: parent_category_english_name,
        } = &self.parent_category
        else {
            return Ok(Category {
                internal_id: self.internal_id,
                english_name: self.english_name.clone(),
                english_description: self.english_description.clone(),
                slovene_name: self.slovene_name.clone(),
                slovene_description: self.slovene_description.clone(),
                parent_category: None,
            });
        };


        let Some(parent_category) = context.category_by_english_name(parent_category_english_name)
        else {
            return Err(
                IntermediateCategoryOutputError::ParentCategoryNotFoundByName {
                    missing_parent_category_english_name: parent_category_english_name.to_owned(),
                },
            );
        };


        Ok(Category {
            // It is crucial that the internal_id is persisted (this is how we look up individual objects, e.g. categories).
            internal_id: self.internal_id,
            english_name: self.english_name.clone(),
            english_description: self.english_description.clone(),
            slovene_name: self.slovene_name.clone(),
            slovene_description: self.slovene_description.clone(),
            parent_category: Some(parent_category.internal_id()),
        })
    }
}




#[derive(Debug, Clone)]
pub struct Category {
    internal_id: InternalCategoryId,

    pub english_name: String,

    // TODO Integrate category descriptions into the backend.
    #[allow(dead_code)]
    pub english_description: Option<String>,

    pub slovene_name: String,

    // TODO Integrate category descriptions into the backend.
    #[allow(dead_code)]
    pub slovene_description: Option<String>,

    parent_category: Option<InternalCategoryId>,
}

impl InternalId for Category {
    type InternalId = InternalCategoryId;

    fn internal_id(&self) -> Self::InternalId {
        self.internal_id
    }
}




#[derive(Debug, Error)]
#[error("unable to find parent category by internal ID: {}", .internal_category_id)]
pub struct ParentCategoryNotFound {
    internal_category_id: InternalCategoryId,
}


impl Category {
    pub fn parent_category<'s>(
        &self,
        growing_category_set: &'s GrowingKeyedSet<Category>,
    ) -> Result<Option<&'s Category>, ParentCategoryNotFound> {
        let Some(internal_parent_category_id) = self.parent_category else {
            return Ok(None);
        };

        growing_category_set
            .get(&internal_parent_category_id)
            .ok_or(ParentCategoryNotFound {
                internal_category_id: internal_parent_category_id,
            })
            .map(Some)
    }
}
