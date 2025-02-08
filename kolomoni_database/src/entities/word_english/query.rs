use chrono::{DateTime, Utc};
use futures_core::stream::BoxStream;
use kolomoni_core::ids::EnglishWordId;
use sqlx::PgConnection;

use super::{
    internal::InternalEnglishWordModel,
    internal_weak::WeakInternalEnglishWordWithMeaningsModel,
    EnglishWordModel,
    EnglishWordWithMeaningsModel,
};
use crate::{
    macros::create_mapped_async_stream,
    IntoExternalModel,
    QueryError,
    QueryResult,
    TryIntoStronglyTypedInternalModel,
};



type RawEnglishWordStream<'c> = BoxStream<'c, Result<InternalEnglishWordModel, sqlx::Error>>;

create_mapped_async_stream!(
    pub struct EnglishWordStream<'c>;
    transforms stream RawEnglishWordStream<'c> => stream of QueryResult<EnglishWordModel>:
        |value|
            value.map(
                |some| some
                    .map(InternalEnglishWordModel::into_external_model)
                    .map_err(|error| QueryError::SqlxError { error })
            )
);


type RawEnglishWordWithMeaningsStream<'c> =
    BoxStream<'c, Result<WeakInternalEnglishWordWithMeaningsModel, sqlx::Error>>;

create_mapped_async_stream!(
    pub struct EnglishWordWithMeaningsStream<'c>;
    transforms stream RawEnglishWordWithMeaningsStream<'c> => stream of QueryResult<EnglishWordWithMeaningsModel>:
        |value| {
            let Some(value) = value else {
                return std::task::Poll::Ready(None);
            };

            let internal_model = value.map_err(|error| QueryError::SqlxError { error })?;

            Some(
                internal_model
                    .try_into_strongly_typed_internal_model()
                    .map_err(QueryError::model_error)
                    .map(IntoExternalModel::into_external_model)
            )
        }
);



#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EnglishWordsQueryOptions {
    /// Ignored if `None` (i.e. no filtering).
    pub only_words_modified_after: Option<DateTime<Utc>>,
}

impl EnglishWordsQueryOptions {
    pub const fn new_without_filters() -> Self {
        Self {
            only_words_modified_after: None,
        }
    }
}


pub struct EnglishWordQuery;

impl EnglishWordQuery {
    pub async fn exists_by_id(
        connection: &mut PgConnection,
        english_word_id: EnglishWordId,
    ) -> QueryResult<bool> {
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS ( \
                SELECT 1 \
                    FROM kolomoni.word_english we \
                    WHERE we.word_id = $1 \
            )",
            english_word_id.into_uuid()
        )
        .fetch_one(connection)
        .await?;

        Ok(exists.unwrap_or(false))
    }

    pub async fn exists_by_exact_lemma(
        connection: &mut PgConnection,
        lemma: &str,
    ) -> QueryResult<bool> {
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS ( \
                SELECT 1 \
                    FROM kolomoni.word_english we \
                    WHERE we.lemma = $1 \
            )",
            lemma
        )
        .fetch_one(connection)
        .await?;

        Ok(exists.unwrap_or(false))
    }

    pub async fn get_by_id(
        connection: &mut PgConnection,
        english_word_id: EnglishWordId,
    ) -> QueryResult<Option<super::EnglishWordModel>> {
        let intermediate_extended_model = sqlx::query_as!(
            InternalEnglishWordModel,
            "SELECT \
                    w.id as \"word_id\", \
                    w.created_at as \"created_at\", \
                    w.last_modified_at as \"last_modified_at\", \
                    we.lemma as \"lemma\" \
                FROM kolomoni.word_english we \
                INNER JOIN kolomoni.word w \
                    ON w.id = we.word_id \
                WHERE we.word_id = $1",
            english_word_id.into_uuid()
        )
        .fetch_optional(connection)
        .await?;

        Ok(intermediate_extended_model.map(InternalEnglishWordModel::into_external_model))
    }

    pub async fn get_by_id_with_meanings(
        connection: &mut PgConnection,
        english_word_id: EnglishWordId,
    ) -> QueryResult<Option<EnglishWordWithMeaningsModel>> {
        let intermediate_extended_model = sqlx::query_file_as!(
            WeakInternalEnglishWordWithMeaningsModel,
            "src/entities/word_english/queries/by_id_including_meanings.sql",
            english_word_id.into_uuid()
        )
        .fetch_optional(connection)
        .await?;

        let Some(intermediate_model) = intermediate_extended_model else {
            return Ok(None);
        };

        Ok(Some(
            intermediate_model
                .try_into_strongly_typed_internal_model()
                .map_err(QueryError::model_error)
                .map(IntoExternalModel::into_external_model)?,
        ))
    }

    pub async fn get_by_exact_lemma(
        connection: &mut PgConnection,
        lemma: &str,
    ) -> QueryResult<Option<super::EnglishWordModel>> {
        let intermediate_extended_model = sqlx::query_as!(
            InternalEnglishWordModel,
            "SELECT \
                    w.id as \"word_id\", \
                    w.created_at as \"created_at\", \
                    w.last_modified_at as \"last_modified_at\", \
                    we.lemma as \"lemma\" \
                FROM kolomoni.word_english we \
                INNER JOIN kolomoni.word w \
                    ON w.id = we.word_id \
                WHERE we.lemma = $1",
            lemma
        )
        .fetch_optional(connection)
        .await?;

        Ok(intermediate_extended_model.map(InternalEnglishWordModel::into_external_model))
    }

    pub async fn get_by_exact_lemma_with_meanings(
        connection: &mut PgConnection,
        lemma: &str,
    ) -> QueryResult<Option<super::EnglishWordWithMeaningsModel>> {
        let intermediate_extended_model = sqlx::query_file_as!(
            WeakInternalEnglishWordWithMeaningsModel,
            "src/entities/word_english/queries/by_lemma_including_meanings.sql",
            lemma
        )
        .fetch_optional(connection)
        .await?;

        let Some(intermediate_model) = intermediate_extended_model else {
            return Ok(None);
        };

        Ok(Some(
            intermediate_model
                .try_into_strongly_typed_internal_model()
                .map_err(QueryError::model_error)
                .map(IntoExternalModel::into_external_model)?,
        ))
    }

    pub async fn get_all_english_words(
        connection: &mut PgConnection,
        options: EnglishWordsQueryOptions,
    ) -> EnglishWordStream<'_> {
        if let Some(only_modified_after) = options.only_words_modified_after {
            let intermediate_word_stream = sqlx::query_as!(
                InternalEnglishWordModel,
                "SELECT \
                        w.id as \"word_id\", \
                        w.created_at as \"created_at\", \
                        w.last_modified_at as \"last_modified_at\", \
                        we.lemma as \"lemma\" \
                    FROM kolomoni.word_english we \
                    INNER JOIN kolomoni.word w \
                        ON w.id = we.word_id \
                    WHERE w.last_modified_at >= $1",
                only_modified_after
            )
            .fetch(connection);

            EnglishWordStream::new(intermediate_word_stream)
        } else {
            let intermediate_word_stream = sqlx::query_as!(
                InternalEnglishWordModel,
                "SELECT \
                        w.id as \"word_id\", \
                        w.created_at as \"created_at\", \
                        w.last_modified_at as \"last_modified_at\", \
                        we.lemma as \"lemma\" \
                    FROM kolomoni.word_english we \
                    INNER JOIN kolomoni.word w \
                        ON w.id = we.word_id",
            )
            .fetch(connection);

            EnglishWordStream::new(intermediate_word_stream)
        }
    }

    pub async fn get_all_english_words_with_meanings(
        database_connection: &mut PgConnection,
        options: EnglishWordsQueryOptions,
    ) -> EnglishWordWithMeaningsStream<'_> {
        if let Some(only_modified_after) = options.only_words_modified_after {
            let internal_words_with_meanings_stream = sqlx::query_file_as!(
                WeakInternalEnglishWordWithMeaningsModel,
                "src/entities/word_english/queries/all_including_meanings_with_last_modified_filter.sql",
                only_modified_after
            )
            .fetch(database_connection);

            EnglishWordWithMeaningsStream::new(internal_words_with_meanings_stream)
        } else {
            let internal_words_with_meanings_stream = sqlx::query_file_as!(
                WeakInternalEnglishWordWithMeaningsModel,
                "src/entities/word_english/queries/all_including_meanings.sql"
            )
            .fetch(database_connection);

            EnglishWordWithMeaningsStream::new(internal_words_with_meanings_stream)
        }
    }
}
