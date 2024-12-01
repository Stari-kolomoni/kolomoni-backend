use parking_lot::MappedRwLockReadGuard;
use thiserror::Error;

use super::{InternalCategoryId, TryToOutputModelWithContext, TryToResolvedModelWithContext};
use crate::{
    analyzer::{
        clean_up_str,
        insert_only_set::{GrowingKeyedSet, InternalId},
    },
    parser::SeedCategoryRow,
};



pub struct IntermediateCategoryResolutionContext<'a> {
    intemediate_categories: &'a GrowingKeyedSet<IntermediateCategory>,
}

impl<'a> IntermediateCategoryResolutionContext<'a> {
    pub fn new(intemediate_categories: &'a GrowingKeyedSet<IntermediateCategory>) -> Self {
        Self {
            intemediate_categories,
        }
    }

    fn category_by_english_name(
        &self,
        english_category_name: &str,
    ) -> Option<MappedRwLockReadGuard<'_, IntermediateCategory>> {
        let mut target_category_internal_id = None;

        for category in self.intemediate_categories.read_inner().values() {
            if english_category_name == category.english_name {
                target_category_internal_id = Some(category.internal_id);
            }
        }

        let Some(target_category_internal_id) = target_category_internal_id else {
            return None;
        };

        self.intemediate_categories
            .get(&target_category_internal_id)
    }
}


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
            english_name: clean_up_str(&row.english_name).to_owned(),
            english_description: row
                .english_description
                .map(|string| clean_up_str(&string).to_owned()),
            slovene_name: clean_up_str(&row.slovene_name).to_owned(),
            slovene_description: row
                .slovene_description
                .map(|string| clean_up_str(&string).to_owned()),
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



#[derive(Debug, Clone)]
pub struct Category {
    internal_id: InternalCategoryId,

    pub english_name: String,
    pub english_description: Option<String>,

    pub slovene_name: String,
    pub slovene_description: Option<String>,

    pub parent_category: Option<InternalCategoryId>,
}

impl InternalId for Category {
    type InternalId = InternalCategoryId;

    fn internal_id(&self) -> Self::InternalId {
        self.internal_id
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
    type Context<'c> = IntermediateCategoryResolutionContext<'c>;
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




pub struct CategoryResolutionContext<'a> {
    categories: &'a GrowingKeyedSet<Category>,
}

impl<'a> CategoryResolutionContext<'a> {
    pub fn new(categories: &'a GrowingKeyedSet<Category>) -> Self {
        Self { categories }
    }

    fn category_by_internal_id(
        &self,
        internal_category_id: &InternalCategoryId,
    ) -> Option<MappedRwLockReadGuard<'a, Category>> {
        self.categories.get(internal_category_id)
    }
}


#[derive(Debug, Clone)]
pub struct ResolvedCategory {
    internal_id: InternalCategoryId,

    pub english_name: String,
    pub english_description: Option<String>,

    pub slovene_name: String,
    pub slovene_description: Option<String>,

    pub parent_category: Option<Box<ResolvedCategory>>,
}


#[derive(Debug, Error)]
pub enum CategoryResolutionError {
    #[error("unable to find parent category by internal ID: {}", .internal_category_id)]
    ParentCategoryNotFound {
        internal_category_id: InternalCategoryId,
    },
}

impl TryToResolvedModelWithContext for Category {
    type ResolvedModel = ResolvedCategory;
    type Context<'ctx> = CategoryResolutionContext<'ctx>;
    type Error = CategoryResolutionError;

    fn try_to_resolved_model<'a>(
        &'a self,
        context: &'a Self::Context<'a>,
    ) -> Result<Self::ResolvedModel, Self::Error> {
        let parent_category = if let Some(parent_category_internal_id) =
            self.parent_category.as_ref()
        {
            let Some(parent_category) = context.category_by_internal_id(parent_category_internal_id)
            else {
                return Err(CategoryResolutionError::ParentCategoryNotFound {
                    internal_category_id: parent_category_internal_id.to_owned(),
                });
            };

            // TODO We might need cycle detection here?
            // (a cycle of parent category relationships can be trivially made, crashing the seeder)
            Some(Box::new(
                parent_category.try_to_resolved_model(context)?,
            ))
        } else {
            None
        };

        Ok(Self::ResolvedModel {
            internal_id: self.internal_id,
            english_name: self.english_name.clone(),
            english_description: self.english_description.clone(),
            slovene_name: self.slovene_name.clone(),
            slovene_description: self.slovene_description.clone(),
            parent_category,
        })
    }
}
