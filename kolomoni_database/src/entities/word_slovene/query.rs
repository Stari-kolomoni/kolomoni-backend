use chrono::{DateTime, Utc};
use futures_core::stream::BoxStream;
use kolomoni_core::ids::SloveneWordId;
use sqlx::PgConnection;

use super::{internal::InternalSloveneWordModel, SloveneWordModel, SloveneWordWithMeaningsModel};
use crate::{
    entities::word_slovene::internal_weak::WeakInternalSloveneWordWithMeaningsModel,
    macros::create_mapped_async_stream,
    IntoExternalModel,
    QueryError,
    QueryResult,
    TryIntoStronglyTypedInternalModel,
};



#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct SloveneWordsQueryOptions {
    /// Ignored if `None` (i.e. no filtering).
    pub only_words_modified_after: Option<DateTime<Utc>>,
}

impl SloveneWordsQueryOptions {
    pub const fn new_without_filters() -> Self {
        Self {
            only_words_modified_after: None,
        }
    }
}



type RawSloveneWordWithMeaningsStream<'c> =
    BoxStream<'c, Result<WeakInternalSloveneWordWithMeaningsModel, sqlx::Error>>;

create_mapped_async_stream!(
    pub struct SloveneWordWithMeaningsStream<'c>;
    transforms stream RawSloveneWordWithMeaningsStream<'c> => stream of QueryResult<SloveneWordWithMeaningsModel>:
        |value| {
            let Some(value) = value else {
                return std::task::Poll::Ready(None);
            };

            let weak_internal_model = value
                .map_err(|error| QueryError::SqlxError { error })?;

            let strong_internal_model = weak_internal_model
                .try_into_strongly_typed_internal_model()
                .map_err(QueryError::model_error)?;

            Some(Ok(strong_internal_model.into_external_model()))
        }
);




pub struct SloveneWordQuery;

impl SloveneWordQuery {
    pub async fn exists_by_id(
        connection: &mut PgConnection,
        slovene_word_id: SloveneWordId,
    ) -> QueryResult<bool> {
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS ( \
                SELECT 1 \
                    FROM kolomoni.word_slovene \
                    WHERE word_id = $1 \
            )",
            slovene_word_id.into_uuid()
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
                    FROM kolomoni.word_slovene \
                    WHERE lemma = $1 \
            )",
            lemma
        )
        .fetch_one(connection)
        .await?;

        Ok(exists.unwrap_or(false))
    }

    pub async fn get_by_id(
        connection: &mut PgConnection,
        slovene_word_id: SloveneWordId,
    ) -> QueryResult<Option<SloveneWordModel>> {
        let intermediate_extended_model = sqlx::query_as!(
            InternalSloveneWordModel,
            "SELECT \
                    word_id, lemma, created_at, last_modified_at \
                FROM kolomoni.word_slovene ws \
                INNER JOIN kolomoni.word w \
                    ON w.id = ws.word_id \
                WHERE ws.word_id = $1",
            slovene_word_id.into_uuid()
        )
        .fetch_optional(connection)
        .await?;

        Ok(intermediate_extended_model.map(IntoExternalModel::into_external_model))
    }

    pub async fn get_by_id_with_meanings(
        database_connection: &mut PgConnection,
        slovene_word_id: SloveneWordId,
    ) -> QueryResult<Option<SloveneWordWithMeaningsModel>> {
        let weak_internal_word_with_meanings = sqlx::query_file_as!(
            WeakInternalSloveneWordWithMeaningsModel,
            "src/entities/word_slovene/queries/by_id_including_meanings.sql",
            slovene_word_id.into_uuid()
        )
        .fetch_optional(database_connection)
        .await?;

        let Some(weak_internal_word_with_meanings) = weak_internal_word_with_meanings else {
            return Ok(None);
        };


        Ok(Some(
            weak_internal_word_with_meanings
                .try_into_strongly_typed_internal_model()
                .map_err(|reason| QueryError::ModelError { reason })?
                .into_external_model(),
        ))
    }

    pub async fn get_by_exact_lemma_with_meanings(
        database_connection: &mut PgConnection,
        lemma: &str,
    ) -> QueryResult<Option<SloveneWordWithMeaningsModel>> {
        let weak_internal_word_with_meanings = sqlx::query_file_as!(
            WeakInternalSloveneWordWithMeaningsModel,
            "src/entities/word_slovene/queries/by_lemma_including_meanings.sql",
            lemma
        )
        .fetch_optional(database_connection)
        .await?;

        let Some(weak_internal_word_with_meanings) = weak_internal_word_with_meanings else {
            return Ok(None);
        };

        Ok(Some(
            weak_internal_word_with_meanings
                .try_into_strongly_typed_internal_model()
                .map_err(|reason| QueryError::ModelError { reason })?
                .into_external_model(),
        ))
    }


    pub async fn get_all_slovene_words_with_meanings(
        database_connection: &mut PgConnection,
        options: SloveneWordsQueryOptions,
    ) -> SloveneWordWithMeaningsStream<'_> {
        if let Some(only_modified_after) = options.only_words_modified_after {
            let weak_filtered_internal_words_with_meanings = sqlx::query_file_as!(
                WeakInternalSloveneWordWithMeaningsModel,
                "src/entities/word_slovene/queries/all_including_meanings_with_last_modified_filter.sql",
                only_modified_after
            )
            .fetch(database_connection);

            SloveneWordWithMeaningsStream::new(weak_filtered_internal_words_with_meanings)
        } else {
            let weak_internal_words_with_meanings = sqlx::query_file_as!(
                WeakInternalSloveneWordWithMeaningsModel,
                "src/entities/word_slovene/queries/all_including_meanings.sql",
            )
            .fetch(database_connection);

            SloveneWordWithMeaningsStream::new(weak_internal_words_with_meanings)
        }
    }
}
