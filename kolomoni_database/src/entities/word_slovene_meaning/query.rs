use kolomoni_core::id::{SloveneWordId, SloveneWordMeaningId};
use sqlx::PgConnection;

use super::SloveneWordMeaningModelWithCategoriesAndTranslations;
use crate::{
    entities::{SloveneWordMeaningModelWithWeaklyTypedCategoriesAndTranslations, WordMeaningQuery},
    IntoExternalModel,
    QueryError,
    QueryResult,
    TryIntoStronglyTypedInternalModel,
};

pub struct SloveneWordMeaningQuery;

impl SloveneWordMeaningQuery {
    pub async fn get_all_by_slovene_word_id(
        database_connection: &mut PgConnection,
        slovene_word_id: SloveneWordId,
    ) -> QueryResult<Vec<SloveneWordMeaningModelWithCategoriesAndTranslations>> {
        let internal_meanings_weak = sqlx::query_as!(
            SloveneWordMeaningModelWithWeaklyTypedCategoriesAndTranslations,
            "SELECT \
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
                WHERE wm.word_id = $1 \
                GROUP BY \
                    wsm.word_meaning_id, \
                    wsm.disambiguation, \
                    wsm.abbreviation, \
                    wsm.description, \
                    wsm.created_at, \
                    wsm.last_modified_at",
            slovene_word_id.into_uuid()
        )
        .fetch_all(database_connection)
        .await?;


        let mut external_meanings = Vec::with_capacity(internal_meanings_weak.len());

        for weak_internal_meaning in internal_meanings_weak {
            let external_meaning = weak_internal_meaning
                .try_into_strongly_typed_internal_model()
                .map_err(|reason| QueryError::ModelError { reason })?
                .into_external_model();

            external_meanings.push(external_meaning);
        }


        Ok(external_meanings)
    }


    pub async fn get(
        database_connection: &mut PgConnection,
        slovene_word_id: SloveneWordId,
        slovene_word_meaning_id: SloveneWordMeaningId,
    ) -> QueryResult<Option<SloveneWordMeaningModelWithCategoriesAndTranslations>> {
        let internal_meaning_weak = sqlx::query_as!(
            SloveneWordMeaningModelWithWeaklyTypedCategoriesAndTranslations,
            "SELECT \
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
                WHERE wm.word_id = $1 AND wm.id = $2 \
                GROUP BY \
                    wsm.word_meaning_id, \
                    wsm.disambiguation, \
                    wsm.abbreviation, \
                    wsm.description, \
                    wsm.created_at, \
                    wsm.last_modified_at",
            slovene_word_id.into_uuid(),
            slovene_word_meaning_id.into_uuid()
        )
        .fetch_optional(database_connection)
        .await?;

        let Some(internal_meaning_weak) = internal_meaning_weak else {
            return Ok(None);
        };


        Ok(Some(
            internal_meaning_weak
                .try_into_strongly_typed_internal_model()
                .map_err(|reason| QueryError::ModelError { reason })?
                .into_external_model(),
        ))
    }


    pub async fn exists_by_id(
        database_connection: &mut PgConnection,
        slovene_word_meaning_id: SloveneWordMeaningId,
    ) -> QueryResult<bool> {
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS (\
                SELECT 1 \
                    FROM kolomoni.word_slovene_meaning \
                    WHERE word_meaning_id = $1
            )",
            slovene_word_meaning_id.into_uuid()
        )
        .fetch_one(database_connection)
        .await?;

        Ok(exists.unwrap_or(false))
    }

    pub async fn exists_by_meaning_and_word_id(
        database_connection: &mut PgConnection,
        slovene_word_id: SloveneWordId,
        slovene_word_meaning_id: SloveneWordMeaningId,
    ) -> QueryResult<bool> {
        WordMeaningQuery::exists_by_meaning_and_word_id(
            database_connection,
            slovene_word_id.into_word_id(),
            slovene_word_meaning_id.into_word_meaning_id(),
        )
        .await
    }
}
