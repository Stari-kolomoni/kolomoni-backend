use miette::{Context, IntoDiagnostic, Result};
use sea_orm::{
    ColumnTrait,
    ConnectionTrait,
    EntityTrait,
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
}
