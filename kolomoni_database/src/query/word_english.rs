use chrono::{DateTime, Utc};
use miette::Result;
use miette::{Context, IntoDiagnostic};
use sea_orm::sea_query::Expr;
use sea_orm::{
    ColumnTrait,
    ConnectionTrait,
    EntityTrait,
    FromQueryResult,
    QueryFilter,
    QuerySelect,
};
use uuid::Uuid;

use super::super::entities::prelude::WordEnglish;
use crate::entities::word_english;


#[derive(Default)]
pub struct EnglishWordsQueryOptions {
    pub only_words_modified_after: Option<DateTime<Utc>>,
}


pub struct EnglishWordQuery;

impl EnglishWordQuery {
    pub async fn word_exists_by_uuid<C: ConnectionTrait>(
        database: &C,
        word_uuid: Uuid,
    ) -> Result<bool> {
        #[derive(Debug, FromQueryResult, PartialEq, Eq, Hash)]
        struct WordCount {
            count: i64,
        }

        let mut word_exists_query = word_english::Entity::find().select_only();

        word_exists_query.expr_as(Expr::val(1).count(), "count");

        let count_result = word_exists_query
            .filter(word_english::Column::WordId.eq(word_uuid))
            .into_model::<WordCount>()
            .one(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while looking up whether the english word exists by uuid.")?;

        match count_result {
            Some(word_count) => {
                debug_assert!(word_count.count <= 1);
                Ok(word_count.count == 1)
            }
            None => Ok(false),
        }
    }

    pub async fn word_exists_by_lemma<C: ConnectionTrait>(
        database: &C,
        lemma: String,
    ) -> Result<bool> {
        #[derive(Debug, FromQueryResult, PartialEq, Eq, Hash)]
        struct WordCount {
            count: i64,
        }

        let mut word_exists_query = word_english::Entity::find().select_only();

        word_exists_query.expr_as(Expr::val(1).count(), "count");

        let count_result = word_exists_query
            .filter(word_english::Column::Lemma.eq(lemma))
            .into_model::<WordCount>()
            .one(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while looking up whether the english word exists by lemma.")?;

        match count_result {
            Some(word_count) => {
                debug_assert!(word_count.count <= 1);
                Ok(word_count.count == 1)
            }
            None => Ok(false),
        }
    }

    pub async fn word_by_uuid<C: ConnectionTrait>(
        database: &C,
        word_uuid: Uuid,
    ) -> Result<Option<word_english::Model>> {
        WordEnglish::find_by_id(word_uuid)
            .one(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while searching database for english word by UUID.")
    }

    pub async fn word_by_lemma<C: ConnectionTrait>(
        database: &C,
        word_lemma: String,
    ) -> Result<Option<word_english::Model>> {
        WordEnglish::find()
            .filter(word_english::Column::Lemma.eq(word_lemma))
            .one(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while searching database for english word by lemma.")
    }

    pub async fn all_words<C: ConnectionTrait>(
        database: &C,
        options: EnglishWordsQueryOptions,
    ) -> Result<Vec<word_english::Model>> {
        let mut query = WordEnglish::find();


        // Add modifiers onto the query based on `options`.
        if let Some(only_words_modified_after) = options.only_words_modified_after {
            query = query.filter(word_english::Column::LastModifiedAt.gt(only_words_modified_after));
        }


        query
            .all(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while querying all english words from the database.")
    }
}
