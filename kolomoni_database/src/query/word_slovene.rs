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
use uuid::Uuid;

use super::{super::entities::prelude::WordSlovene, WordCategoryQuery};
use crate::entities::{category, word_slovene};


#[derive(Default)]
pub struct SloveneWordsQueryOptions {
    pub only_words_modified_after: Option<DateTime<Utc>>,
}


#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RelatedSloveneWordInfo {
    pub categories: Vec<category::Model>,
}


#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExpandedSloveneWordInfo {
    pub word: word_slovene::Model,
    pub categories: Vec<category::Model>,
}



pub struct SloveneWordQuery;

impl SloveneWordQuery {
    pub async fn word_exists_by_uuid<C: ConnectionTrait + TransactionTrait>(
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

    pub async fn word_exists_by_lemma<C: ConnectionTrait + TransactionTrait>(
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

    pub async fn word_by_uuid<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        word_uuid: Uuid,
    ) -> Result<Option<word_slovene::Model>> {
        WordSlovene::find_by_id(word_uuid)
            .one(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while searching database for slovene word by UUID.")
    }

    pub async fn word_by_lemma<C: ConnectionTrait + TransactionTrait>(
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

    pub async fn expanded_word_by_uuid<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        word_uuid: Uuid,
    ) -> Result<Option<ExpandedSloveneWordInfo>> {
        let optional_base_word = WordSlovene::find_by_id(word_uuid)
            .one(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while searching database for slovene word by UUID.")?;

        let Some(base_word) = optional_base_word else {
            return Ok(None);
        };


        let related_info = Self::related_word_information_only(database, word_uuid).await?;


        Ok(Some(ExpandedSloveneWordInfo {
            word: base_word,
            categories: related_info.categories,
        }))
    }

    pub async fn expanded_word_by_lemma<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        word_lemma: String,
    ) -> Result<Option<ExpandedSloveneWordInfo>> {
        let optional_base_word = WordSlovene::find()
            .filter(word_slovene::Column::Lemma.eq(word_lemma))
            .one(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while seaching database for slovene word by lemma.")?;

        let Some(base_word) = optional_base_word else {
            return Ok(None);
        };


        let related_info = Self::related_word_information_only(database, base_word.word_id).await?;


        Ok(Some(ExpandedSloveneWordInfo {
            word: base_word,
            categories: related_info.categories,
        }))
    }

    pub async fn all_words<C: ConnectionTrait + TransactionTrait>(
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

    pub async fn all_words_expanded<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        options: SloveneWordsQueryOptions,
    ) -> Result<Vec<ExpandedSloveneWordInfo>> {
        let mut query = WordSlovene::find();

        // Add modifiers onto the query based on `options`.
        if let Some(only_words_modified_after) = options.only_words_modified_after {
            query = query.filter(word_slovene::Column::LastModifiedAt.gt(only_words_modified_after));
        }

        let base_words = query
            .all(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while querying all slovene words from the database.")?;


        // PERF This could be improved (but the expanded english word has bigger issues).

        let mut expanded_slovene_words = Vec::with_capacity(base_words.len());

        for base_slovene_word in base_words {
            let related_info =
                Self::related_word_information_only(database, base_slovene_word.word_id).await?;

            expanded_slovene_words.push(ExpandedSloveneWordInfo {
                word: base_slovene_word,
                categories: related_info.categories,
            });
        }

        Ok(expanded_slovene_words)
    }


    /// PERF: This might be a good candidate for optimization, probably with caching.
    pub async fn related_word_information_only<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        word_uuid: Uuid,
    ) -> Result<RelatedSloveneWordInfo> {
        let categories =
            WordCategoryQuery::word_categories_by_word_uuid(database, word_uuid).await?;

        Ok(RelatedSloveneWordInfo { categories })
    }
}
