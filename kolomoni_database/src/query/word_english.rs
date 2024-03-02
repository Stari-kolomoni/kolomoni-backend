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
    TransactionTrait,
};
use uuid::Uuid;

use super::super::entities::prelude::WordEnglish;
use super::{
    ExpandedSloveneWordInfo,
    TranslationQuery,
    TranslationSuggestionQuery,
    WordCategoryQuery,
};
use crate::entities::{category, word_english};


#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct EnglishWordsQueryOptions {
    pub only_words_modified_after: Option<DateTime<Utc>>,
}


pub struct RelatedEnglishWordInfo {
    pub categories: Vec<category::Model>,
    pub suggested_translations: Vec<ExpandedSloveneWordInfo>,
    pub translations: Vec<ExpandedSloveneWordInfo>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExpandedEnglishWordInfo {
    pub word: word_english::Model,
    pub categories: Vec<category::Model>,
    pub suggested_translations: Vec<ExpandedSloveneWordInfo>,
    pub translations: Vec<ExpandedSloveneWordInfo>,
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

    pub async fn all_words_expanded<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        options: EnglishWordsQueryOptions,
    ) -> Result<Vec<ExpandedEnglishWordInfo>> {
        let mut query = WordEnglish::find();


        // Add modifiers onto the query based on `options`.
        if let Some(only_words_modified_after) = options.only_words_modified_after {
            query = query.filter(word_english::Column::LastModifiedAt.gt(only_words_modified_after));
        }


        let base_words = query
            .all(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while querying all english words from the database.")?;


        // PERF This could be improved.

        let mut expanded_english_words = Vec::with_capacity(base_words.len());

        for base_english_word in base_words {
            let related_info =
                Self::related_word_information_only(database, base_english_word.word_id).await?;

            expanded_english_words.push(ExpandedEnglishWordInfo {
                word: base_english_word,
                categories: related_info.categories,
                suggested_translations: related_info.suggested_translations,
                translations: related_info.translations,
            });
        }

        Ok(expanded_english_words)
    }

    pub async fn expanded_word_by_uuid<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        word_uuid: Uuid,
    ) -> Result<Option<ExpandedEnglishWordInfo>> {
        let Some(word_model) = Self::word_by_uuid(database, word_uuid).await? else {
            return Ok(None);
        };

        let related_info = Self::related_word_information_only(database, word_uuid).await?;

        Ok(Some(ExpandedEnglishWordInfo {
            word: word_model,
            categories: related_info.categories,
            suggested_translations: related_info.suggested_translations,
            translations: related_info.translations,
        }))
    }

    pub async fn expanded_word_by_lemma<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        word_lemma: String,
    ) -> Result<Option<ExpandedEnglishWordInfo>> {
        let Some(word_model) = Self::word_by_lemma(database, word_lemma).await? else {
            return Ok(None);
        };

        let related_info = Self::related_word_information_only(database, word_model.word_id).await?;

        Ok(Some(ExpandedEnglishWordInfo {
            word: word_model,
            categories: related_info.categories,
            suggested_translations: related_info.suggested_translations,
            translations: related_info.translations,
        }))
    }

    /// PERF: This might be a good candidate for optimization, probably with caching.
    pub async fn related_word_information_only<C: ConnectionTrait + TransactionTrait>(
        database: &C,
        word_uuid: Uuid,
    ) -> Result<RelatedEnglishWordInfo> {
        let categories =
            WordCategoryQuery::word_categories_by_word_uuid(database, word_uuid).await?;


        let suggested_translations = {
            let suggested_translation_models =
                TranslationSuggestionQuery::suggestions_for_english_word(database, word_uuid)
                    .await?;


            let mut suggested_translations = Vec::with_capacity(suggested_translation_models.len());
            for suggested_translation_model in suggested_translation_models {
                let suggested_translation_word_categories =
                    WordCategoryQuery::word_categories_by_word_uuid(
                        database,
                        suggested_translation_model.word_id,
                    )
                    .await?;

                suggested_translations.push(ExpandedSloveneWordInfo {
                    word: suggested_translation_model,
                    categories: suggested_translation_word_categories,
                });
            }

            suggested_translations
        };


        let translations = {
            let translation_models =
                TranslationQuery::translations_for_english_word(database, word_uuid).await?;


            let mut translations = Vec::with_capacity(translation_models.len());
            for translation_model in translation_models {
                let translated_word_categories = WordCategoryQuery::word_categories_by_word_uuid(
                    database,
                    translation_model.word_id,
                )
                .await?;

                translations.push(ExpandedSloveneWordInfo {
                    word: translation_model,
                    categories: translated_word_categories,
                });
            }

            translations
        };


        Ok(RelatedEnglishWordInfo {
            categories,
            suggested_translations,
            translations,
        })
    }
}
