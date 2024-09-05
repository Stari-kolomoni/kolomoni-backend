use futures_core::stream::BoxStream;
use sqlx::PgConnection;

use super::SloveneWordId;
use crate::{IntoModel, QueryError, QueryResult};


type RawSloveneWordStream<'c> = BoxStream<'c, Result<super::IntermediateExtendedModel, sqlx::Error>>;

create_async_stream_wrapper!(
    pub struct SloveneWordStream<'c>;
    transforms stream RawSloveneWordStream<'c> => stream of QueryResult<super::ExtendedModel>:
        |value|
            value.map(
                |some| some
                    .map(super::IntermediateExtendedModel::into_model)
                    .map_err(|error| QueryError::SqlxError { error })
            )
);



pub struct Query;

impl Query {
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
            slovene_word_id.into_inner()
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
    ) -> QueryResult<Option<super::ExtendedModel>> {
        let intermediate_extended_model = sqlx::query_as!(
            super::IntermediateExtendedModel,
            "SELECT word_id, lemma, created_at, last_modified_at \
                FROM kolomoni.word_slovene \
                INNER JOIN kolomoni.word \
                    ON word.id = word_slovene.word_id \
                WHERE word_slovene.word_id = $1",
            slovene_word_id.into_inner()
        )
        .fetch_optional(connection)
        .await?;

        Ok(intermediate_extended_model.map(super::IntermediateExtendedModel::into_model))
    }

    pub async fn get_by_exact_lemma(
        connection: &mut PgConnection,
        lemma: &str,
    ) -> QueryResult<Option<super::ExtendedModel>> {
        let intermediate_extended_model = sqlx::query_as!(
            super::IntermediateExtendedModel,
            "SELECT word_id, lemma, created_at, last_modified_at \
                FROM kolomoni.word_slovene \
                INNER JOIN kolomoni.word \
                    ON word.id = word_slovene.word_id \
                WHERE word_slovene.lemma = $1",
            lemma
        )
        .fetch_optional(connection)
        .await?;

        Ok(intermediate_extended_model.map(super::IntermediateExtendedModel::into_model))
    }

    pub async fn get_all_slovene_words(connection: &mut PgConnection) -> SloveneWordStream<'_> {
        let intermediate_word_stream = sqlx::query_as!(
            super::IntermediateExtendedModel,
            "SELECT word_id, lemma, created_at, last_modified_at \
                FROM kolomoni.word_slovene \
                INNER JOIN kolomoni.word \
                    ON word.id = word_slovene.word_id"
        )
        .fetch(connection);

        SloveneWordStream::new(intermediate_word_stream)
    }
}
