use chrono::{DateTime, Utc};
use futures_core::stream::BoxStream;
use kolomoni_core::id::SloveneWordId;
use sqlx::PgConnection;

use super::SloveneWordWithMeaningsModel;
use crate::{IntoExternalModel, QueryError, QueryResult, TryIntoExternalModel};



#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct SloveneWordsQueryOptions {
    /// Ignored if `None` (i.e. no filtering).
    pub only_words_modified_after: Option<DateTime<Utc>>,
}


type RawSloveneWordStream<'c> = BoxStream<'c, Result<super::InternalSloveneWordModel, sqlx::Error>>;

create_async_stream_wrapper!(
    pub struct SloveneWordStream<'c>;
    transforms stream RawSloveneWordStream<'c> => stream of QueryResult<super::SloveneWordModel>:
        |value|
            value.map(
                |some| some
                    .map(super::InternalSloveneWordModel::into_external_model)
                    .map_err(|error| QueryError::SqlxError { error })
            )
);



type RawSloveneWordWithMeaningsStream<'c> =
    BoxStream<'c, Result<super::InternalSloveneWordWithMeaningsModel, sqlx::Error>>;

create_async_stream_wrapper!(
    pub struct SloveneWordWithMeaningsStream<'c>;
    transforms stream RawSloveneWordWithMeaningsStream<'c> => stream of QueryResult<super::SloveneWordWithMeaningsModel>:
        |value| {
            let Some(value) = value else {
                return std::task::Poll::Ready(None);
            };

            let internal_model = value.map_err(|error| QueryError::SqlxError { error })?;

            Some(
                internal_model.try_into_external_model()
                    .map_err(|reason| QueryError::ModelError { reason })
            )
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
    ) -> QueryResult<Option<super::SloveneWordModel>> {
        let intermediate_extended_model = sqlx::query_as!(
            super::InternalSloveneWordModel,
            "SELECT word_id, lemma, created_at, last_modified_at \
                FROM kolomoni.word_slovene \
                INNER JOIN kolomoni.word \
                    ON word.id = word_slovene.word_id \
                WHERE word_slovene.word_id = $1",
            slovene_word_id.into_uuid()
        )
        .fetch_optional(connection)
        .await?;

        Ok(intermediate_extended_model.map(super::InternalSloveneWordModel::into_external_model))
    }

    pub async fn get_by_id_with_meanings(
        database_connection: &mut PgConnection,
        slovene_word_id: SloveneWordId,
    ) -> QueryResult<Option<SloveneWordWithMeaningsModel>> {
        let internal_word_with_meanings = sqlx::query_as!(
            super::InternalSloveneWordWithMeaningsModel,
            "SELECT \
                    ws.word_id         as \"word_id\", \
                    ws.lemma           as \"lemma\", \
                    w.created_at       as \"created_at\", \
                    w.last_modified_at as \"last_modified_at\", \
                    coalesce( \
                        json_agg(meanings) \
                            FILTER (WHERE meanings.word_meaning_id IS NOT NULL), \
                        '[]'::json \
                    ) as \"meanings!\" \
                FROM kolomoni.word_slovene as ws \
                INNER JOIN kolomoni.word as w \
                    ON ws.word_id =  w.id \
                LEFT JOIN LATERAL ( \
                    SELECT \
                            wsm.word_meaning_id as \"word_meaning_id\", \
                            wsm.disambiguation as \"disambiguation\", \
                            wsm.abbreviation as \"abbreviation\", \
                            wsm.description as \"description\", \
                            wsm.created_at as \"created_at\", \
                            wsm.last_modified_at as \"last_modified_at\", \
                            coalesce( \
                                json_agg(categories) \
                                    FILTER (WHERE categories.category_id IS NOT NULL), \
                                '[]'::json \
                            ) as \"categories\", \
                            coalesce( \
                                json_agg(translates_into) \
                                    FILTER (WHERE translates_into.translated_at IS NOT NULL), \
                                '[]'::json \
                            ) as \"translates_into\" \
                        FROM kolomoni.word_slovene_meaning as wsm \
                        INNER JOIN kolomoni.word_meaning as wm \
                            ON wsm.word_meaning_id = wm.id \
                        LEFT JOIN LATERAL ( \
                            SELECT wec.category_id as \"category_id\" \
                                FROM kolomoni.word_meaning_category wec \
                                WHERE wec.word_meaning_id = wsm.word_meaning_id \
                        ) categories ON TRUE \
                        LEFT JOIN LATERAL ( \
                            SELECT \
                                wem.word_meaning_id  as \"meaning_id\", \
                                wem.description      as \"description\", \
                                wem.disambiguation   as \"disambiguation\", \
                                wem.abbreviation     as \"abbreviation\", \
                                wem.created_at       as \"created_at\", \
                                wem.last_modified_at as \"last_modified_at\", \
                                coalesce( \
                                    json_agg(categories_on_translated) \
                                        FILTER (WHERE categories_on_translated.category_id IS NOT NULL), \
                                    '[]'::json \
                                ) as \"categories\", \
                                translated_at, \
                                translated_by \
                            FROM kolomoni.word_meaning_translation wmt \
                                INNER JOIN kolomoni.word_english_meaning as wem \
                                        ON wmt.english_word_meaning_id = wem.word_meaning_id \
                                LEFT JOIN LATERAL ( \
                                    SELECT wec_t.category_id as \"category_id\" \
                                    FROM kolomoni.word_meaning_category wec_t \
                                    WHERE wec_t.word_meaning_id = wem.word_meaning_id \
                                ) categories_on_translated ON TRUE \
                            WHERE wmt.slovene_word_meaning_id = wm.id \
                            GROUP BY \
                                wem.word_meaning_id, \
                                wem.description, \
                                wem.disambiguation, \
                                wem.abbreviation, \
                                wem.created_at, \
                                wem.last_modified_at, \
                                wmt.translated_at, \
                                wmt.translated_by \
                        ) translates_into ON TRUE \
                        WHERE wm.word_id = ws.word_id \
                        GROUP BY \
                            wsm.word_meaning_id, \
                            wsm.disambiguation, \
                            wsm.abbreviation, \
                            wsm.description, \
                            wsm.created_at, \
                            wsm.last_modified_at \
                ) meanings ON TRUE \
                WHERE ws.word_id = $1 \
                GROUP BY \
                    ws.word_id, \
                    ws.lemma, \
                    w.created_at, \
                    w.last_modified_at",
            slovene_word_id.into_uuid()
        )
        .fetch_optional(database_connection).await?;


        let Some(internal_model) = internal_word_with_meanings else {
            return Ok(None);
        };


        Ok(Some(
            internal_model
                .try_into_external_model()
                .map_err(|reason| QueryError::ModelError { reason })?,
        ))
    }

    pub async fn get_by_exact_lemma(
        connection: &mut PgConnection,
        lemma: &str,
    ) -> QueryResult<Option<super::SloveneWordModel>> {
        let intermediate_extended_model = sqlx::query_as!(
            super::InternalSloveneWordModel,
            "SELECT word_id, lemma, created_at, last_modified_at \
                FROM kolomoni.word_slovene \
                INNER JOIN kolomoni.word \
                    ON word.id = word_slovene.word_id \
                WHERE word_slovene.lemma = $1",
            lemma
        )
        .fetch_optional(connection)
        .await?;

        Ok(intermediate_extended_model.map(super::InternalSloveneWordModel::into_external_model))
    }

    pub async fn get_by_exact_lemma_with_meanings(
        database_connection: &mut PgConnection,
        lemma: &str,
    ) -> QueryResult<Option<SloveneWordWithMeaningsModel>> {
        let internal_word_with_meanings = sqlx::query_as!(
            super::InternalSloveneWordWithMeaningsModel,
            "SELECT \
                    ws.word_id         as \"word_id\", \
                    ws.lemma           as \"lemma\", \
                    w.created_at       as \"created_at\", \
                    w.last_modified_at as \"last_modified_at\", \
                    coalesce( \
                        json_agg(meanings) \
                            FILTER (WHERE meanings.word_meaning_id IS NOT NULL), \
                        '[]'::json \
                    ) as \"meanings!\" \
                FROM kolomoni.word_slovene as ws \
                INNER JOIN kolomoni.word as w \
                    ON ws.word_id =  w.id \
                LEFT JOIN LATERAL ( \
                    SELECT \
                            wsm.word_meaning_id as \"word_meaning_id\", \
                            wsm.disambiguation as \"disambiguation\", \
                            wsm.abbreviation as \"abbreviation\", \
                            wsm.description as \"description\", \
                            wsm.created_at as \"created_at\", \
                            wsm.last_modified_at as \"last_modified_at\", \
                            coalesce( \
                                json_agg(categories) \
                                    FILTER (WHERE categories.category_id IS NOT NULL), \
                                '[]'::json \
                            ) as \"categories\", \
                            coalesce( \
                                json_agg(translates_into) \
                                    FILTER (WHERE translates_into.translated_at IS NOT NULL), \
                                '[]'::json \
                            ) as \"translates_into\" \
                        FROM kolomoni.word_slovene_meaning as wsm \
                        INNER JOIN kolomoni.word_meaning as wm \
                            ON wsm.word_meaning_id = wm.id \
                        LEFT JOIN LATERAL ( \
                            SELECT wec.category_id as \"category_id\" \
                                FROM kolomoni.word_meaning_category wec \
                                WHERE wec.word_meaning_id = wsm.word_meaning_id \
                        ) categories ON TRUE \
                        LEFT JOIN LATERAL ( \
                            SELECT \
                                wem.word_meaning_id  as \"meaning_id\", \
                                wem.description      as \"description\", \
                                wem.disambiguation   as \"disambiguation\", \
                                wem.abbreviation     as \"abbreviation\", \
                                wem.created_at       as \"created_at\", \
                                wem.last_modified_at as \"last_modified_at\", \
                                coalesce( \
                                    json_agg(categories_on_translated) \
                                        FILTER (WHERE categories_on_translated.category_id IS NOT NULL), \
                                    '[]'::json \
                                ) as \"categories\", \
                                translated_at, \
                                translated_by \
                            FROM kolomoni.word_meaning_translation wmt \
                                INNER JOIN kolomoni.word_english_meaning as wem \
                                        ON wmt.english_word_meaning_id = wem.word_meaning_id \
                                LEFT JOIN LATERAL ( \
                                    SELECT wec_t.category_id as \"category_id\" \
                                    FROM kolomoni.word_meaning_category wec_t \
                                    WHERE wec_t.word_meaning_id = wem.word_meaning_id \
                                ) categories_on_translated ON TRUE \
                            WHERE wmt.slovene_word_meaning_id = wm.id \
                            GROUP BY \
                                wem.word_meaning_id, \
                                wem.description, \
                                wem.disambiguation, \
                                wem.abbreviation, \
                                wem.created_at, \
                                wem.last_modified_at, \
                                wmt.translated_at, \
                                wmt.translated_by \
                        ) translates_into ON TRUE \
                        WHERE wm.word_id = ws.word_id \
                        GROUP BY \
                            wsm.word_meaning_id, \
                            wsm.disambiguation, \
                            wsm.abbreviation, \
                            wsm.description, \
                            wsm.created_at, \
                            wsm.last_modified_at \
                ) meanings ON TRUE \
                WHERE ws.lemma = $1 \
                GROUP BY \
                    ws.word_id, \
                    ws.lemma, \
                    w.created_at, \
                    w.last_modified_at",
            lemma
        )
        .fetch_optional(database_connection).await?;


        let Some(internal_model) = internal_word_with_meanings else {
            return Ok(None);
        };


        Ok(Some(
            internal_model
                .try_into_external_model()
                .map_err(|reason| QueryError::ModelError { reason })?,
        ))
    }

    pub async fn get_all_slovene_words(connection: &mut PgConnection) -> SloveneWordStream<'_> {
        let intermediate_word_stream = sqlx::query_as!(
            super::InternalSloveneWordModel,
            "SELECT word_id, lemma, created_at, last_modified_at \
                FROM kolomoni.word_slovene \
                INNER JOIN kolomoni.word \
                    ON word.id = word_slovene.word_id"
        )
        .fetch(connection);

        SloveneWordStream::new(intermediate_word_stream)
    }

    pub async fn get_all_slovene_words_with_meanings(
        database_connection: &mut PgConnection,
        options: SloveneWordsQueryOptions,
    ) -> SloveneWordWithMeaningsStream<'_> {
        if let Some(only_modified_after) = options.only_words_modified_after {
            let internal_words_with_meanings_stream = sqlx::query_as!(
                super::InternalSloveneWordWithMeaningsModel,
                "SELECT \
                        ws.word_id         as \"word_id\", \
                        ws.lemma           as \"lemma\", \
                        w.created_at       as \"created_at\", \
                        w.last_modified_at as \"last_modified_at\", \
                        coalesce( \
                            json_agg(meanings) \
                                FILTER (WHERE meanings.word_meaning_id IS NOT NULL), \
                            '[]'::json \
                        ) as \"meanings!\" \
                    FROM kolomoni.word_slovene as ws \
                    INNER JOIN kolomoni.word as w \
                        ON ws.word_id =  w.id \
                    LEFT JOIN LATERAL ( \
                        SELECT \
                                wsm.word_meaning_id as \"word_meaning_id\", \
                                wsm.disambiguation as \"disambiguation\", \
                                wsm.abbreviation as \"abbreviation\", \
                                wsm.description as \"description\", \
                                wsm.created_at as \"created_at\", \
                                wsm.last_modified_at as \"last_modified_at\", \
                                coalesce( \
                                    json_agg(categories) \
                                        FILTER (WHERE categories.category_id IS NOT NULL), \
                                    '[]'::json \
                                ) as \"categories\", \
                                coalesce( \
                                    json_agg(translates_into) \
                                        FILTER (WHERE translates_into.translated_at IS NOT NULL), \
                                    '[]'::json \
                                ) as \"translates_into\" \
                            FROM kolomoni.word_slovene_meaning as wsm \
                            INNER JOIN kolomoni.word_meaning as wm \
                                ON wsm.word_meaning_id = wm.id \
                            LEFT JOIN LATERAL ( \
                                SELECT wec.category_id as \"category_id\" \
                                    FROM kolomoni.word_meaning_category wec \
                                    WHERE wec.word_meaning_id = wsm.word_meaning_id \
                            ) categories ON TRUE \
                            LEFT JOIN LATERAL ( \
                                SELECT \
                                    wem.word_meaning_id  as \"meaning_id\", \
                                    wem.description      as \"description\", \
                                    wem.disambiguation   as \"disambiguation\", \
                                    wem.abbreviation     as \"abbreviation\", \
                                    wem.created_at       as \"created_at\", \
                                    wem.last_modified_at as \"last_modified_at\", \
                                    coalesce( \
                                        json_agg(categories_on_translated) \
                                            FILTER (WHERE categories_on_translated.category_id IS NOT NULL), \
                                        '[]'::json \
                                    ) as \"categories\", \
                                    translated_at, \
                                    translated_by \
                                FROM kolomoni.word_meaning_translation wmt \
                                    INNER JOIN kolomoni.word_english_meaning as wem \
                                            ON wmt.english_word_meaning_id = wem.word_meaning_id \
                                    LEFT JOIN LATERAL ( \
                                        SELECT wec_t.category_id as \"category_id\" \
                                        FROM kolomoni.word_meaning_category wec_t \
                                        WHERE wec_t.word_meaning_id = wem.word_meaning_id \
                                    ) categories_on_translated ON TRUE \
                                WHERE wmt.slovene_word_meaning_id = wm.id \
                                GROUP BY \
                                    wem.word_meaning_id, \
                                    wem.description, \
                                    wem.disambiguation, \
                                    wem.abbreviation, \
                                    wem.created_at, \
                                    wem.last_modified_at, \
                                    wmt.translated_at, \
                                    wmt.translated_by \
                            ) translates_into ON TRUE \
                            WHERE wm.word_id = ws.word_id \
                            GROUP BY \
                                wsm.word_meaning_id, \
                                wsm.disambiguation, \
                                wsm.abbreviation, \
                                wsm.description, \
                                wsm.created_at, \
                                wsm.last_modified_at \
                    ) meanings ON TRUE \
                    WHERE w.last_modified_at >= $1 \
                    GROUP BY \
                        ws.word_id, \
                        ws.lemma, \
                        w.created_at, \
                        w.last_modified_at",
                    only_modified_after
            )
            .fetch(database_connection);

            SloveneWordWithMeaningsStream::new(internal_words_with_meanings_stream)
        } else {
            let internal_words_with_meanings_stream = sqlx::query_as!(
                super::InternalSloveneWordWithMeaningsModel,
                "SELECT \
                        ws.word_id         as \"word_id\", \
                        ws.lemma           as \"lemma\", \
                        w.created_at       as \"created_at\", \
                        w.last_modified_at as \"last_modified_at\", \
                        coalesce( \
                            json_agg(meanings) \
                                FILTER (WHERE meanings.word_meaning_id IS NOT NULL), \
                            '[]'::json \
                        ) as \"meanings!\" \
                    FROM kolomoni.word_slovene as ws \
                    INNER JOIN kolomoni.word as w \
                        ON ws.word_id =  w.id \
                    LEFT JOIN LATERAL ( \
                        SELECT \
                                wsm.word_meaning_id as \"word_meaning_id\", \
                                wsm.disambiguation as \"disambiguation\", \
                                wsm.abbreviation as \"abbreviation\", \
                                wsm.description as \"description\", \
                                wsm.created_at as \"created_at\", \
                                wsm.last_modified_at as \"last_modified_at\", \
                                coalesce( \
                                    json_agg(categories) \
                                        FILTER (WHERE categories.category_id IS NOT NULL), \
                                    '[]'::json \
                                ) as \"categories\", \
                                coalesce( \
                                    json_agg(translates_into) \
                                        FILTER (WHERE translates_into.translated_at IS NOT NULL), \
                                    '[]'::json \
                                ) as \"translates_into\" \
                            FROM kolomoni.word_slovene_meaning as wsm \
                            INNER JOIN kolomoni.word_meaning as wm \
                                ON wsm.word_meaning_id = wm.id \
                            LEFT JOIN LATERAL ( \
                                SELECT wec.category_id as \"category_id\" \
                                    FROM kolomoni.word_meaning_category wec \
                                    WHERE wec.word_meaning_id = wsm.word_meaning_id \
                            ) categories ON TRUE \
                            LEFT JOIN LATERAL ( \
                                SELECT \
                                    wem.word_meaning_id  as \"meaning_id\", \
                                    wem.description      as \"description\", \
                                    wem.disambiguation   as \"disambiguation\", \
                                    wem.abbreviation     as \"abbreviation\", \
                                    wem.created_at       as \"created_at\", \
                                    wem.last_modified_at as \"last_modified_at\", \
                                    coalesce( \
                                        json_agg(categories_on_translated) \
                                            FILTER (WHERE categories_on_translated.category_id IS NOT NULL), \
                                        '[]'::json \
                                    ) as \"categories\", \
                                    translated_at, \
                                    translated_by \
                                FROM kolomoni.word_meaning_translation wmt \
                                    INNER JOIN kolomoni.word_english_meaning as wem \
                                            ON wmt.english_word_meaning_id = wem.word_meaning_id \
                                    LEFT JOIN LATERAL ( \
                                        SELECT wec_t.category_id as \"category_id\" \
                                        FROM kolomoni.word_meaning_category wec_t \
                                        WHERE wec_t.word_meaning_id = wem.word_meaning_id \
                                    ) categories_on_translated ON TRUE \
                                WHERE wmt.slovene_word_meaning_id = wm.id \
                                GROUP BY \
                                    wem.word_meaning_id, \
                                    wem.description, \
                                    wem.disambiguation, \
                                    wem.abbreviation, \
                                    wem.created_at, \
                                    wem.last_modified_at, \
                                    wmt.translated_at, \
                                    wmt.translated_by \
                            ) translates_into ON TRUE \
                            WHERE wm.word_id = ws.word_id \
                            GROUP BY \
                                wsm.word_meaning_id, \
                                wsm.disambiguation, \
                                wsm.abbreviation, \
                                wsm.description, \
                                wsm.created_at, \
                                wsm.last_modified_at \
                    ) meanings ON TRUE \
                    GROUP BY \
                        ws.word_id, \
                        ws.lemma, \
                        w.created_at, \
                        w.last_modified_at"
            )
            .fetch(database_connection);

            SloveneWordWithMeaningsStream::new(internal_words_with_meanings_stream)
        }
    }
}
