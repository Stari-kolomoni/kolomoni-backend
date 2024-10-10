use std::borrow::Cow;

use chrono::Utc;
use kolomoni_core::ids::CategoryId;
use sqlx::{PgConnection, Postgres, QueryBuilder};

use super::CategoryModel;
use crate::{IntoExternalModel, QueryError, QueryResult};



#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NewCategory {
    pub parent_category_id: Option<CategoryId>,
    pub slovene_name: String,
    pub english_name: String,
}



#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CategoryValuesToUpdate {
    pub parent_category_id: Option<Option<CategoryId>>,
    pub slovene_name: Option<String>,
    pub english_name: Option<String>,
}

impl CategoryValuesToUpdate {
    fn has_any_values_to_update(&self) -> bool {
        self.parent_category_id.is_some()
            || self.slovene_name.is_some()
            || self.english_name.is_some()
    }
}


fn build_category_update_query(
    category_id: CategoryId,
    values_to_update: CategoryValuesToUpdate,
) -> QueryBuilder<'static, Postgres> {
    let mut update_query_builder = QueryBuilder::new("UPDATE kolomoni.category SET ");

    let mut separated_set_expressions = update_query_builder.separated(", ");

    if let Some(new_parent_category_id) = values_to_update.parent_category_id {
        separated_set_expressions.push_unseparated("parent_category_id = ");
        separated_set_expressions.push_bind(new_parent_category_id.map(|id| id.into_uuid()));
    }

    if let Some(new_slovene_name) = values_to_update.slovene_name {
        separated_set_expressions.push_unseparated("name_sl = ");
        separated_set_expressions.push_bind(new_slovene_name);
    }

    if let Some(new_english_name) = values_to_update.english_name {
        separated_set_expressions.push_unseparated("name_en = ");
        separated_set_expressions.push_bind(new_english_name);
    }


    update_query_builder.push(" WHERE id = ");
    update_query_builder.push_bind(category_id.into_uuid());

    update_query_builder
}




pub struct CategoryMutation;

impl CategoryMutation {
    pub async fn create(
        database_connection: &mut PgConnection,
        new_category: NewCategory,
    ) -> QueryResult<CategoryModel> {
        let new_category_id = CategoryId::generate();
        let new_category_created_at = Utc::now();
        let new_category_last_modified_at = new_category_created_at;

        let newly_created_category = sqlx::query_as!(
            super::InternalCategoryModel,
            "INSERT INTO kolomoni.category \
                (id, parent_category_id, name_sl, name_en, \
                 created_at, last_modified_at) \
                VALUES ($1, $2, $3, $4, $5, $6) \
                RETURNING \
                    id, parent_category_id, name_sl, name_en, \
                    created_at, last_modified_at",
            new_category_id.into_uuid(),
            new_category.parent_category_id.map(|id| id.into_uuid()),
            new_category.slovene_name,
            new_category.english_name,
            new_category_created_at,
            new_category_last_modified_at
        )
        .fetch_one(database_connection)
        .await?;

        Ok(newly_created_category.into_external_model())
    }


    pub async fn update(
        database_connection: &mut PgConnection,
        category_id: CategoryId,
        category_values_to_update: CategoryValuesToUpdate,
    ) -> QueryResult<bool> {
        if !category_values_to_update.has_any_values_to_update() {
            return Ok(true);
        }


        let mut update_query_builder =
            build_category_update_query(category_id, category_values_to_update);

        let query_result = update_query_builder
            .build()
            .execute(database_connection)
            .await?;


        Ok(query_result.rows_affected() == 1)
    }

    pub async fn delete(
        database_connection: &mut PgConnection,
        category_id: CategoryId,
    ) -> QueryResult<bool> {
        let query_result = sqlx::query!(
            "DELETE FROM kolomoni.category \
                WHERE id = $1",
            category_id.into_uuid()
        )
        .execute(database_connection)
        .await?;


        if query_result.rows_affected() > 1 {
            return Err(QueryError::DatabaseInconsistencyError {
                problem: Cow::from(
                    "attempted to delete a category by ID, but more than one row matched",
                ),
            });
        }

        Ok(query_result.rows_affected() == 1)
    }
}
