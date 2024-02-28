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
};
use uuid::Uuid;

use super::super::entities::prelude::WordSlovene;
use crate::entities::word_slovene;


#[derive(Default)]
pub struct SloveneWordsQueryOptions {
    pub only_words_modified_after: Option<DateTime<Utc>>,
}


pub struct SloveneWordQuery;

impl SloveneWordQuery {
    pub async fn word_exists_by_uuid<C: ConnectionTrait>(
        database: &C,
        word_uuid: Uuid,
    ) -> Result<bool> {
        #[derive(Debug, FromQueryResult, PartialEq, Eq, Hash)]
        struct WordCount {
            count: i64,
        }

        let mut word_exists_query = word_slovene::Entity::find().select_only();

        word_exists_query.expr_as(Expr::val(1).count(), "count");

        let count_result = word_exists_query
            .filter(word_slovene::Column::WordId.eq(word_uuid))
            .into_model::<WordCount>()
            .one(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while looking up whether the slovene word exists by uuid.")?;

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

        let mut word_exists_query = word_slovene::Entity::find().select_only();

        word_exists_query.expr_as(Expr::val(1).count(), "count");

        let count_result = word_exists_query
            .filter(word_slovene::Column::Lemma.eq(lemma))
            .into_model::<WordCount>()
            .one(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while looking up whether the slovene word exists by lemma.")?;

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
    ) -> Result<Option<word_slovene::Model>> {
        WordSlovene::find_by_id(word_uuid)
            .one(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while searching database for slovene word by UUID.")
    }

    pub async fn word_by_lemma<C: ConnectionTrait>(
        database: &C,
        word_lemma: String,
    ) -> Result<Option<word_slovene::Model>> {
        WordSlovene::find()
            .filter(word_slovene::Column::Lemma.eq(word_lemma))
            .one(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while seaching database for slovene word by lemma.")
    }

    pub async fn all_words<C: ConnectionTrait>(
        database: &C,
        options: SloveneWordsQueryOptions,
    ) -> Result<Vec<word_slovene::Model>> {
        let mut query = WordSlovene::find();

        // Add modifiers onto the query based on `options`.
        if let Some(only_words_modified_after) = options.only_words_modified_after {
            query = query.filter(word_slovene::Column::LastModifiedAt.gt(only_words_modified_after));
        }

        query
            .all(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while querying all slovene words from the database.")
    }
}
