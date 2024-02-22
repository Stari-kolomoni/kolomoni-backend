use miette::{miette, Context, IntoDiagnostic, Result};
use sea_orm::{ActiveModelTrait, ActiveValue, ConnectionTrait, TransactionTrait};

use crate::entities::category;


#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NewCategory {
    pub name: String,
}

pub struct CategoryMutation;

impl CategoryMutation {
    pub async fn create<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        category: NewCategory,
    ) -> Result<category::Model> {
        let active_category = category::ActiveModel {
            name: ActiveValue::Set(category.name),
            ..Default::default()
        };

        let new_category = active_category
            .insert(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed to insert category into the database.")?;

        Ok(new_category)
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
            Err(miette!("Failed to delete category from the database: no such database."))
        }
    }

    // TODO
}
