use chrono::{DateTime, Utc};
use futures_core::stream::BoxStream;
use kolomoni_core::id::EnglishWordId;
use sqlx::PgConnection;

use crate::{IntoExternalModel, QueryError, QueryResult, TryIntoExternalModel};



type RawEnglishWordStream<'c> = BoxStream<'c, Result<super::InternalEnglishWordModel, sqlx::Error>>;

create_async_stream_wrapper!(
    pub struct EnglishWordStream<'c>;
    transforms stream RawEnglishWordStream<'c> => stream of QueryResult<super::EnglishWordModel>:
        |value|
            value.map(
                |some| some
                    .map(super::InternalEnglishWordModel::into_external_model)
                    .map_err(|error| QueryError::SqlxError { error })
            )
);


type RawEnglishWordWithMeaningsStream<'c> =
    BoxStream<'c, Result<super::InternalEnglishWordWithMeaningsModel, sqlx::Error>>;

create_async_stream_wrapper!(
    pub struct EnglishWordWithMeaningsStream<'c>;
    transforms stream RawEnglishWordWithMeaningsStream<'c> => stream of QueryResult<super::EnglishWordWithMeaningsModel>:
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



#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct EnglishWordsQueryOptions {
    /// Ignored if `None` (i.e. no filtering).
    pub only_words_modified_after: Option<DateTime<Utc>>,
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
                    FROM kolomoni.word_english \
                    WHERE word_id = $1 \
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
                    FROM kolomoni.word_english \
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
        english_word_id: EnglishWordId,
    ) -> QueryResult<Option<super::EnglishWordModel>> {
        let intermediate_extended_model = sqlx::query_as!(
            super::InternalEnglishWordModel,
            "SELECT word_id, lemma, created_at, last_modified_at \
                FROM kolomoni.word_english \
                INNER JOIN kolomoni.word \
                    ON word.id = word_english.word_id \
                WHERE word_english.word_id = $1",
            english_word_id.into_uuid()
        )
        .fetch_optional(connection)
        .await?;

        Ok(intermediate_extended_model.map(super::InternalEnglishWordModel::into_external_model))
    }

    pub async fn get_by_id_with_meanings(
        connection: &mut PgConnection,
        english_word_id: EnglishWordId,
    ) -> QueryResult<Option<super::EnglishWordWithMeaningsModel>> {
        let intermediate_extended_model = sqlx::query_as!(
            super::InternalEnglishWordWithMeaningsModel,
            "SELECT \
                    we.word_id as \"word_id\", \
                    we.lemma as \"lemma\", \
                    w.created_at as \"created_at\", \
                    w.last_modified_at as \"last_modified_at\", \
                    coalesce( \
                        json_agg(meanings) \
                            FILTER (WHERE meanings.word_meaning_id IS NOT NULL), \
                        '[]'::json \
                    ) as \"meanings!\" \
                FROM kolomoni.word_english as we \
                INNER JOIN kolomoni.word as w \
                    ON we.word_id =  w.id \
                LEFT JOIN LATERAL ( \
                    SELECT \
                            wem.word_meaning_id as \"word_meaning_id\", \
                            wem.disambiguation as \"disambiguation\", \
                            wem.abbreviation as \"abbreviation\", \
                            wem.description as \"description\", \
                            wem.created_at as \"created_at\", \
                            wem.last_modified_at as \"last_modified_at\", \
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
                        FROM kolomoni.word_english_meaning as wem \
                        INNER JOIN kolomoni.word_meaning as wm \
                            ON wem.word_meaning_id = wm.id \
                        LEFT JOIN LATERAL ( \
                            SELECT wec.category_id as \"category_id\" \
                                FROM kolomoni.word_meaning_category wec \
                                WHERE wec.word_meaning_id = wem.word_meaning_id \
                        ) categories ON TRUE \
                        LEFT JOIN LATERAL ( \
                            SELECT \
                                wsm.word_meaning_id as \"word_meaning_id\", \
                                wsm.description as \"description\", \
                                wsm.disambiguation as \"disambiguation\", \
                                wsm.abbreviation as \"abbreviation\", \
                                wsm.created_at as \"created_at\", \
                                wsm.last_modified_at as \"last_modified_at\", \
                                coalesce( \
                                    json_agg(categories_on_translated) \
                                        FILTER (WHERE categories_on_translated.category_id IS NOT NULL), \
                                    '[]'::json \
                                ) as \"categories\", \
                                translated_at, \
                                translated_by \
                                FROM kolomoni.word_meaning_translation wmt \
                                INNER JOIN kolomoni.word_slovene_meaning as wsm \
                                    ON wmt.slovene_word_meaning_id = wsm.word_meaning_id \
                                LEFT JOIN LATERAL ( \
                                        SELECT wec_t.category_id as \"category_id\" \
                                            FROM kolomoni.word_meaning_category wec_t \
                                            WHERE wec_t.word_meaning_id = wsm.word_meaning_id \
                                    ) categories_on_translated ON TRUE \
                                WHERE wmt.english_word_meaning_id = wm.id \
                                GROUP BY \
                                    wsm.word_meaning_id, \
                                    wsm.description, \
                                    wsm.disambiguation, \
                                    wsm.abbreviation, \
                                    wsm.created_at, \
                                    wsm.last_modified_at, \
                                    wmt.translated_at, \
                                    wmt.translated_by \
                        ) translates_into ON TRUE \
                        WHERE wm.word_id = we.word_id \
                        GROUP BY \
                            wem.word_meaning_id, \
                            wem.disambiguation, \
                            wem.abbreviation, \
                            wem.description, \
                            wem.created_at, \
                            wem.last_modified_at \
                ) meanings ON TRUE \
                WHERE we.word_id = $1
                GROUP BY \
                    we.word_id, \
                    we.lemma, \
                    w.created_at, \
                    w.last_modified_at",
            english_word_id.into_uuid()
        )
        .fetch_optional(connection)
        .await?;


        let Some(intermediate_model) = intermediate_extended_model else {
            return Ok(None);
        };

        Ok(Some(
            intermediate_model
                .try_into_external_model()
                .map_err(|reason| QueryError::ModelError { reason })?,
        ))
    }

    pub async fn get_by_exact_lemma(
        connection: &mut PgConnection,
        lemma: &str,
    ) -> QueryResult<Option<super::EnglishWordModel>> {
        let intermediate_extended_model = sqlx::query_as!(
            super::InternalEnglishWordModel,
            "SELECT word_id, lemma, created_at, last_modified_at \
                FROM kolomoni.word_english \
                INNER JOIN kolomoni.word \
                    ON word.id = word_english.word_id \
                WHERE word_english.lemma = $1",
            lemma
        )
        .fetch_optional(connection)
        .await?;

        Ok(intermediate_extended_model.map(super::InternalEnglishWordModel::into_external_model))
    }

    pub async fn get_by_exact_lemma_with_meanings(
        connection: &mut PgConnection,
        lemma: &str,
    ) -> QueryResult<Option<super::EnglishWordWithMeaningsModel>> {
        let intermediate_extended_model = sqlx::query_as!(
            super::InternalEnglishWordWithMeaningsModel,
            "SELECT \
                    we.word_id as \"word_id\", \
                    we.lemma as \"lemma\", \
                    w.created_at as \"created_at\", \
                    w.last_modified_at as \"last_modified_at\", \
                    coalesce( \
                        json_agg(meanings) \
                            FILTER (WHERE meanings.word_meaning_id IS NOT NULL), \
                        '[]'::json \
                    ) as \"meanings!\" \
                FROM kolomoni.word_english as we \
                INNER JOIN kolomoni.word as w \
                    ON we.word_id =  w.id \
                LEFT JOIN LATERAL ( \
                    SELECT \
                            wem.word_meaning_id as \"word_meaning_id\", \
                            wem.disambiguation as \"disambiguation\", \
                            wem.abbreviation as \"abbreviation\", \
                            wem.description as \"description\", \
                            wem.created_at as \"created_at\", \
                            wem.last_modified_at as \"last_modified_at\", \
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
                        FROM kolomoni.word_english_meaning as wem \
                        INNER JOIN kolomoni.word_meaning as wm \
                            ON wem.word_meaning_id = wm.id \
                        LEFT JOIN LATERAL ( \
                            SELECT wec.category_id as \"category_id\" \
                                FROM kolomoni.word_meaning_category wec \
                                WHERE wec.word_meaning_id = wem.word_meaning_id \
                        ) categories ON TRUE \
                        LEFT JOIN LATERAL ( \
                            SELECT \
                                wsm.word_meaning_id as \"word_meaning_id\", \
                                wsm.description as \"description\", \
                                wsm.disambiguation as \"disambiguation\", \
                                wsm.abbreviation as \"abbreviation\", \
                                wsm.created_at as \"created_at\", \
                                wsm.last_modified_at as \"last_modified_at\", \
                                coalesce( \
                                    json_agg(categories_on_translated) \
                                        FILTER (WHERE categories_on_translated.category_id IS NOT NULL), \
                                    '[]'::json \
                                ) as \"categories\", \
                                translated_at, \
                                translated_by \
                                FROM kolomoni.word_meaning_translation wmt \
                                INNER JOIN kolomoni.word_slovene_meaning as wsm \
                                    ON wmt.slovene_word_meaning_id = wsm.word_meaning_id \
                                LEFT JOIN LATERAL ( \
                                        SELECT wec_t.category_id as \"category_id\" \
                                            FROM kolomoni.word_meaning_category wec_t \
                                            WHERE wec_t.word_meaning_id = wsm.word_meaning_id \
                                    ) categories_on_translated ON TRUE \
                                WHERE wmt.english_word_meaning_id = wm.id \
                                GROUP BY \
                                    wsm.word_meaning_id, \
                                    wsm.description, \
                                    wsm.disambiguation, \
                                    wsm.abbreviation, \
                                    wsm.created_at, \
                                    wsm.last_modified_at, \
                                    wmt.translated_at, \
                                    wmt.translated_by \
                        ) translates_into ON TRUE \
                        WHERE wm.word_id = we.word_id \
                        GROUP BY \
                            wem.word_meaning_id, \
                            wem.disambiguation, \
                            wem.abbreviation, \
                            wem.description, \
                            wem.created_at, \
                            wem.last_modified_at \
                ) meanings ON TRUE \
                WHERE we.lemma = $1 \
                GROUP BY \
                    we.word_id, \
                    we.lemma, \
                    w.created_at, \
                    w.last_modified_at",
            lemma
        )
        .fetch_optional(connection)
        .await?;


        let Some(intermediate_model) = intermediate_extended_model else {
            return Ok(None);
        };

        Ok(Some(
            intermediate_model
                .try_into_external_model()
                .map_err(|reason| QueryError::ModelError { reason })?,
        ))
    }

    pub async fn get_all_english_words(
        connection: &mut PgConnection,
        options: EnglishWordsQueryOptions,
    ) -> EnglishWordStream<'_> {
        if let Some(only_modified_after) = options.only_words_modified_after {
            let intermediate_word_stream = sqlx::query_as!(
                super::InternalEnglishWordModel,
                "SELECT word_id, lemma, created_at, last_modified_at \
                    FROM kolomoni.word_english \
                    INNER JOIN kolomoni.word \
                        ON word.id = word_english.word_id \
                    WHERE last_modified_at >= $1",
                only_modified_after
            )
            .fetch(connection);

            EnglishWordStream::new(intermediate_word_stream)
        } else {
            let intermediate_word_stream = sqlx::query_as!(
                super::InternalEnglishWordModel,
                "SELECT word_id, lemma, created_at, last_modified_at \
                    FROM kolomoni.word_english \
                    INNER JOIN kolomoni.word \
                        ON word.id = word_english.word_id"
            )
            .fetch(connection);

            EnglishWordStream::new(intermediate_word_stream)
        }
    }

    // TODO Needs to be tested.
    pub async fn get_all_english_words_with_meanings(
        database_connection: &mut PgConnection,
        options: EnglishWordsQueryOptions,
    ) -> EnglishWordWithMeaningsStream<'_> {
        if let Some(only_modified_after) = options.only_words_modified_after {
            let internal_words_with_meanings_stream = sqlx::query_as!(
                super::InternalEnglishWordWithMeaningsModel,
                "SELECT \
                        we.word_id as \"word_id\", \
                        we.lemma as \"lemma\", \
                        w.created_at as \"created_at\", \
                        w.last_modified_at as \"last_modified_at\", \
                        coalesce( \
                            json_agg(meanings) \
                            FILTER (WHERE meanings.word_meaning_id IS NOT NULL), \
                            '[]'::json \
                        ) as \"meanings!\" \
                    FROM kolomoni.word_english as we \
                    INNER JOIN kolomoni.word as w \
                        ON we.word_id =  w.id \
                    LEFT JOIN LATERAL ( \
                        SELECT \
                                wem.word_meaning_id as \"word_meaning_id\", \
                                wem.disambiguation as \"disambiguation\", \
                                wem.abbreviation as \"abbreviation\", \
                                wem.description as \"description\", \
                                wem.created_at as \"created_at\", \
                                wem.last_modified_at as \"last_modified_at\", \
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
                            FROM kolomoni.word_english_meaning as wem \
                            INNER JOIN kolomoni.word_meaning as wm \
                                ON wem.word_meaning_id = wm.id \
                            LEFT JOIN LATERAL ( \
                                SELECT wec.category_id as \"category_id\" \
                                    FROM kolomoni.word_meaning_category wec \
                                    WHERE wec.word_meaning_id = wem.word_meaning_id \
                            ) categories ON TRUE \
                            LEFT JOIN LATERAL ( \
                                SELECT \
                                    wsm.word_meaning_id as \"word_meaning_id\", \
                                    wsm.description as \"description\", \
                                    wsm.disambiguation as \"disambiguation\", \
                                    wsm.abbreviation as \"abbreviation\", \
                                    wsm.created_at as \"created_at\", \
                                    wsm.last_modified_at as \"last_modified_at\", \
                                    coalesce( \
                                        json_agg(categories_on_translated) \
                                            FILTER (WHERE categories_on_translated.category_id IS NOT NULL), \
                                        '[]'::json \
                                    ) as \"categories\", \
                                    translated_at, \
                                    translated_by \
                                    FROM kolomoni.word_meaning_translation wmt \
                                    INNER JOIN kolomoni.word_slovene_meaning as wsm \
                                        ON wmt.slovene_word_meaning_id = wsm.word_meaning_id \
                                    LEFT JOIN LATERAL ( \
                                        SELECT wec_t.category_id as \"category_id\" \
                                            FROM kolomoni.word_meaning_category wec_t \
                                            WHERE wec_t.word_meaning_id = wsm.word_meaning_id \
                                    ) categories_on_translated ON TRUE \
                                    WHERE wmt.english_word_meaning_id = wm.id \
                                    GROUP BY \
                                        wsm.word_meaning_id, \
                                        wsm.description, \
                                        wsm.disambiguation, \
                                        wsm.abbreviation, \
                                        wsm.created_at, \
                                        wsm.last_modified_at, \
                                        wmt.translated_at, \
                                        wmt.translated_by \
                            ) translates_into ON TRUE \
                            WHERE wm.word_id = we.word_id \
                            GROUP BY \
                                wem.word_meaning_id, \
                                wem.disambiguation, \
                                wem.abbreviation, \
                                wem.description, \
                                wem.created_at, \
                                wem.last_modified_at \
                    ) meanings ON TRUE \
                    WHERE w.last_modified_at >= $1 \
                    GROUP BY \
                        we.word_id, \
                        we.lemma, \
                        w.created_at, \
                        w.last_modified_at",
                only_modified_after
            )
            .fetch(database_connection);

            EnglishWordWithMeaningsStream::new(internal_words_with_meanings_stream)
        } else {
            let internal_words_with_meanings_stream = sqlx::query_as!(
                super::InternalEnglishWordWithMeaningsModel,
                "SELECT \
                        we.word_id as \"word_id\", \
                        we.lemma as \"lemma\", \
                        w.created_at as \"created_at\", \
                        w.last_modified_at as \"last_modified_at\", \
                        coalesce( \
                            json_agg(meanings) \
                            FILTER (WHERE meanings.word_meaning_id IS NOT NULL), \
                            '[]'::json \
                        ) as \"meanings!\" \
                    FROM kolomoni.word_english as we \
                    INNER JOIN kolomoni.word as w \
                        ON we.word_id =  w.id \
                    LEFT JOIN LATERAL ( \
                        SELECT \
                                wem.word_meaning_id as \"word_meaning_id\", \
                                wem.disambiguation as \"disambiguation\", \
                                wem.abbreviation as \"abbreviation\", \
                                wem.description as \"description\", \
                                wem.created_at as \"created_at\", \
                                wem.last_modified_at as \"last_modified_at\", \
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
                            FROM kolomoni.word_english_meaning as wem \
                            INNER JOIN kolomoni.word_meaning as wm \
                                ON wem.word_meaning_id = wm.id \
                            LEFT JOIN LATERAL ( \
                                SELECT wec.category_id as \"category_id\" \
                                    FROM kolomoni.word_meaning_category wec \
                                    WHERE wec.word_meaning_id = wem.word_meaning_id \
                            ) categories ON TRUE \
                            LEFT JOIN LATERAL ( \
                                SELECT \
                                    wsm.word_meaning_id as \"word_meaning_id\", \
                                    wsm.description as \"description\", \
                                    wsm.disambiguation as \"disambiguation\", \
                                    wsm.abbreviation as \"abbreviation\", \
                                    wsm.created_at as \"created_at\", \
                                    wsm.last_modified_at as \"last_modified_at\", \
                                    coalesce( \
                                        json_agg(categories_on_translated) \
                                            FILTER (WHERE categories_on_translated.category_id IS NOT NULL), \
                                        '[]'::json \
                                    ) as \"categories\", \
                                    translated_at, \
                                    translated_by \
                                    FROM kolomoni.word_meaning_translation wmt \
                                    INNER JOIN kolomoni.word_slovene_meaning as wsm \
                                        ON wmt.slovene_word_meaning_id = wsm.word_meaning_id \
                                    LEFT JOIN LATERAL ( \
                                        SELECT wec_t.category_id as \"category_id\" \
                                            FROM kolomoni.word_meaning_category wec_t \
                                            WHERE wec_t.word_meaning_id = wsm.word_meaning_id \
                                    ) categories_on_translated ON TRUE \
                                    WHERE wmt.english_word_meaning_id = wm.id \
                                    GROUP BY \
                                        wsm.word_meaning_id, \
                                        wsm.description, \
                                        wsm.disambiguation, \
                                        wsm.abbreviation, \
                                        wsm.created_at, \
                                        wsm.last_modified_at, \
                                        wmt.translated_at, \
                                        wmt.translated_by \
                            ) translates_into ON TRUE \
                            WHERE wm.word_id = we.word_id \
                            GROUP BY \
                                wem.word_meaning_id, \
                                wem.disambiguation, \
                                wem.abbreviation, \
                                wem.description, \
                                wem.created_at, \
                                wem.last_modified_at \
                    ) meanings ON TRUE \
                    GROUP BY \
                        we.word_id, \
                        we.lemma, \
                        w.created_at, \
                        w.last_modified_at",
            )
            .fetch(database_connection);

            EnglishWordWithMeaningsStream::new(internal_words_with_meanings_stream)
        }
    }
}
