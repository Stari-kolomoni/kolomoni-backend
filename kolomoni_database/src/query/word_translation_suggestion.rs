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

use crate::entities::{word_slovene, word_translation_suggestion};

pub struct TranslationSuggestionQuery;

impl TranslationSuggestionQuery {
    pub async fn suggestions_for_english_word<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        english_word_uuid: Uuid,
    ) -> Result<Vec<word_slovene::Model>> {
        let suggestions_query = word_slovene::Entity::find()
            .inner_join(word_translation_suggestion::Entity)
            .filter(word_translation_suggestion::Column::EnglishWordId.eq(english_word_uuid))
            .all(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while retrieving suggested translations from database.")?;

        Ok(suggestions_query)
    }

    pub async fn exists<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        english_word_uuid: Uuid,
        slovene_word_uuid: Uuid,
    ) -> Result<bool> {
        #[derive(Debug, FromQueryResult, PartialEq, Eq, Hash)]
        struct SuggestionCount {
            count: i64,
        }


        let mut suggestions_query = word_slovene::Entity::find()
            .inner_join(word_translation_suggestion::Entity)
            .filter(word_translation_suggestion::Column::EnglishWordId.eq(english_word_uuid))
            .filter(word_translation_suggestion::Column::SloveneWordId.eq(slovene_word_uuid))
            .select_only();

        suggestions_query.expr_as(Expr::val(1).count(), "count");

        let suggestion_count_result = suggestions_query
            .into_model::<SuggestionCount>()
            .one(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while looking whether a suggested translation exists in database.")?;


        match suggestion_count_result {
            Some(suggestion_count) => {
                debug_assert!(suggestion_count.count <= 1);
                Ok(suggestion_count.count == 1)
            }
            None => Ok(false),
        }
    }
}
