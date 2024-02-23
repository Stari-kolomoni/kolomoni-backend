use miette::{Context, IntoDiagnostic, Result};
use sea_orm::{
    sea_query::Expr,
    ColumnTrait,
    ConnectionTrait,
    EntityTrait,
    FromQueryResult,
    JoinType,
    QueryFilter,
    QuerySelect,
    TransactionTrait,
};
use uuid::Uuid;

use crate::entities::{category, word_category};

pub struct WordCategoryQuery;

impl WordCategoryQuery {
    pub async fn word_categories_by_word_uuid<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        word_uuid: Uuid,
    ) -> Result<Vec<category::Model>> {
        let select_query = category::Entity::find()
            .join(
                JoinType::InnerJoin,
                category::Entity::belongs_to(word_category::Entity)
                    .from(category::Column::Id)
                    .to(word_category::Column::CategoryId)
                    .into(),
            )
            .filter(word_category::Column::WordId.eq(word_uuid))
            .all(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while looking up word categories by word UUID.")?;

        Ok(select_query)
    }

    pub async fn word_has_category<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        word_uuid: Uuid,
        category_id: i32,
    ) -> Result<bool> {
        #[derive(Debug, FromQueryResult, PartialEq, Eq, Hash)]
        struct CountResult {
            count: i64,
        }

        let mut query = word_category::Entity::find().select_only();

        query.expr_as(Expr::val(1).count(), "count");

        let count_result = query
            .filter(word_category::Column::WordId.eq(word_uuid))
            .filter(word_category::Column::CategoryId.eq(category_id))
            .into_model::<CountResult>()
            .one(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while looking up whether a word has a category.")?;


        match count_result {
            Some(count) => Ok(count.count == 1),
            None => Ok(false),
        }
    }
}
