use chrono::{DateTime, Utc};
use miette::{Context, IntoDiagnostic, Result};
use sea_orm::{
    sea_query::Expr,
    ColumnTrait,
    ConnectionTrait,
    EntityTrait,
    FromQueryResult,
    QueryFilter,
    QuerySelect,
    TransactionTrait,
};

use crate::entities::category;


#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct CategoriesQueryOptions {
    pub only_categories_modified_after: Option<DateTime<Utc>>,
}


pub struct CategoryQuery;

impl CategoryQuery {
    pub async fn exists_by_id<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        category_id: i32,
    ) -> Result<bool> {
        #[derive(Debug, FromQueryResult, PartialEq, Eq, Hash)]
        struct SuggestionCount {
            count: i64,
        }

        let mut select_query = category::Entity::find()
            .filter(category::Column::Id.eq(category_id))
            .select_only();

        select_query.expr_as(Expr::val(1).count(), "count");

        let select_result = select_query
            .into_model::<SuggestionCount>()
            .one(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed whie looking up whether a category ID exists in the database.")?;


        match select_result {
            Some(count) => {
                debug_assert!(count.count <= 1);
                Ok(count.count == 1)
            }
            None => Ok(false),
        }
    }

    pub async fn exists_by_both_names<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        slovene_name: String,
        english_name: String,
    ) -> Result<bool> {
        #[derive(Debug, FromQueryResult, PartialEq, Eq, Hash)]
        struct SuggestionCount {
            count: i64,
        }

        let mut select_query = category::Entity::find()
            .filter(category::Column::SloveneName.eq(slovene_name))
            .filter(category::Column::EnglishName.eq(english_name))
            .select_only();

        select_query.expr_as(Expr::val(1).count(), "count");

        let select_result = select_query
            .into_model::<SuggestionCount>()
            .one(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed whie looking up whether a category name exists in the database.")?;


        match select_result {
            Some(count) => {
                debug_assert!(count.count <= 1);
                Ok(count.count == 1)
            }
            None => Ok(false),
        }
    }

    pub async fn get_by_id<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        category_id: i32,
    ) -> Result<Option<category::Model>> {
        let query = category::Entity::find_by_id(category_id)
            .one(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while fetching category from database.")?;

        Ok(query)
    }

    pub async fn all<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        options: CategoriesQueryOptions,
    ) -> Result<Vec<category::Model>> {
        let mut query = category::Entity::find();

        if let Some(only_categories_modified_after) = options.only_categories_modified_after {
            query =
                query.filter(category::Column::LastModifiedAt.gt(only_categories_modified_after));
        }


        let categories = query
            .all(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while fetching all categories from database.")?;

        Ok(categories)
    }
}
