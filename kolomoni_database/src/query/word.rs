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
use uuid::Uuid;

use crate::entities::word;

pub struct WordQuery;

impl WordQuery {
    pub async fn exists_by_uuid<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        word_uuid: Uuid,
    ) -> Result<bool> {
        #[derive(Debug, FromQueryResult, PartialEq, Eq, Hash)]
        struct CountResult {
            count: i64,
        }

        let mut query = word::Entity::find().select_only();

        query.expr_as(Expr::val(1).count(), "count");

        let count_result = query
            .filter(word::Column::Id.eq(word_uuid))
            .into_model::<CountResult>()
            .one(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while looking up whether a word exists by UUID.")?;


        match count_result {
            Some(count) => Ok(count.count == 1),
            None => Ok(false),
        }
    }
}
