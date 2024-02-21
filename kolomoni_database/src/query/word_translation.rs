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

use crate::entities::{word_slovene, word_translation};

pub struct TranslationQuery;

impl TranslationQuery {
    pub async fn translations_for_english_word<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        english_word_uuid: Uuid,
    ) -> Result<Vec<word_slovene::Model>> {
        let translations_query = word_slovene::Entity::find()
            .inner_join(word_translation::Entity)
            .filter(word_translation::Column::EnglishWordId.eq(english_word_uuid))
            .all(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while retrieving translations from database.")?;

        Ok(translations_query)
    }

    pub async fn exists<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        english_word_uuid: Uuid,
        slovene_word_uuid: Uuid,
    ) -> Result<bool> {
        #[derive(Debug, FromQueryResult, PartialEq, Eq, Hash)]
        struct TranslationCount {
            count: i64,
        }


        let mut suggestions_query = word_slovene::Entity::find()
            .inner_join(word_translation::Entity)
            .filter(word_translation::Column::EnglishWordId.eq(english_word_uuid))
            .filter(word_translation::Column::SloveneWordId.eq(slovene_word_uuid))
            .select_only();

        suggestions_query.expr_as(Expr::val(1).count(), "count");

        let translation_count_result = suggestions_query
            .into_model::<TranslationCount>()
            .one(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while looking whether a translation exists in database.")?;


        match translation_count_result {
            Some(translation_count) => {
                debug_assert!(translation_count.count <= 1);
                Ok(translation_count.count == 1)
            }
            None => Ok(false),
        }
    }
}
