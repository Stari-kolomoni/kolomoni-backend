use miette::{miette, Context, IntoDiagnostic, Result};
use sea_orm::{ActiveModelTrait, ActiveValue, ConnectionTrait, TransactionTrait};

use crate::entities::category;


#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NewCategory {
    pub slovene_name: String,
    pub english_name: String,
}


#[derive(Clone, PartialEq, Eq, Debug)]
pub struct UpdatedCategory {
    pub slovene_name: Option<String>,
    pub english_name: Option<String>,
}


pub struct CategoryMutation;

impl CategoryMutation {
    pub async fn create<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        category: NewCategory,
    ) -> Result<category::Model> {
        let active_category = category::ActiveModel {
            slovene_name: ActiveValue::Set(category.slovene_name),
            english_name: ActiveValue::Set(category.english_name),
            ..Default::default()
        };

        let new_category = active_category
            .insert(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed to insert category into the database.")?;

        Ok(new_category)
    }

    pub async fn update<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        category_id: i32,
        update: UpdatedCategory,
    ) -> Result<category::Model> {
        let mut active_category = category::ActiveModel {
            id: ActiveValue::Unchanged(category_id),
            ..Default::default()
        };

        if let Some(updated_slovene_name) = update.slovene_name {
            active_category.slovene_name = ActiveValue::Set(updated_slovene_name);
        }

        if let Some(updated_english_name) = update.english_name {
            active_category.english_name = ActiveValue::Set(updated_english_name);
        }


        let updated_category = active_category
            .update(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while updating category in database.")?;

        Ok(updated_category)
    }

    pub async fn delete<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        category_id: i32,
    ) -> Result<()> {
        let active_category = category::ActiveModel {
            id: ActiveValue::Unchanged(category_id),
            ..Default::default()
        };

        let deletion_result = active_category
            .delete(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed to delete category from the database.")?;


        if deletion_result.rows_affected == 1 {
            Ok(())
        } else {
            Err(miette!(
                "Failed to delete category from the database: no such database."
            ))
        }
    }
}
