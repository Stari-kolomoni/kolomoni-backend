use kolomoni_core::ids::{SloveneWordId, SloveneWordMeaningId};
use sqlx::PgConnection;

use super::SloveneWordMeaningModelWithDetails;
use crate::{
    entities::{
        word_meaning::WordMeaningQuery,
        word_meaning_slovene::internal_weak::WeakInternalSloveneWordMeaningModelWithDetails,
    },
    IntoExternalModel,
    QueryError,
    QueryResult,
    TryIntoStronglyTypedInternalModel,
};


pub struct SloveneWordMeaningLookup {
    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,
}



pub struct SloveneWordMeaningQuery;

impl SloveneWordMeaningQuery {
    pub async fn get_all_by_slovene_word_id(
        database_connection: &mut PgConnection,
        slovene_word_id: SloveneWordId,
    ) -> QueryResult<Vec<SloveneWordMeaningModelWithDetails>> {
        let weak_internal_meanings = sqlx::query_file_as!(
            WeakInternalSloveneWordMeaningModelWithDetails,
            "src/entities/word_meaning_slovene/queries/all_meanings_for_given_slovene_word.sql",
            slovene_word_id.into_uuid()
        )
        .fetch_all(database_connection)
        .await?;


        let mut external_meanings = Vec::with_capacity(weak_internal_meanings.len());

        for weak_internal_meaning in weak_internal_meanings {
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
    ) -> QueryResult<Option<SloveneWordMeaningModelWithDetails>> {
        let weak_internal_meaning = sqlx::query_file_as!(
            WeakInternalSloveneWordMeaningModelWithDetails,
            "src/entities/word_meaning_slovene/queries/single_meaning_by_both_ids.sql",
            slovene_word_id.into_uuid(),
            slovene_word_meaning_id.into_uuid()
        )
        .fetch_optional(database_connection)
        .await?;

        let Some(weak_internal_meaning) = weak_internal_meaning else {
            return Ok(None);
        };


        Ok(Some(
            weak_internal_meaning
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
                    FROM kolomoni.word_meaning_slovene AS wms \
                    WHERE wms.word_meaning_id = $1
            )",
            slovene_word_meaning_id.into_uuid()
        )
        .fetch_one(database_connection)
        .await?;

        Ok(exists.unwrap_or(false))
    }

    pub async fn exists_by_distinguishing_fields(
        database_connection: &mut PgConnection,
        distinguishing_fields: SloveneWordMeaningLookup,
    ) -> QueryResult<bool> {
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS (\
                SELECT 1 \
                    FROM kolomoni.word_meaning_slovene AS wms \
                    WHERE \
                        wms.disambiguation = $1 \
                        AND wms.abbreviation = $2 \
                        AND wms.description = $3
            )",
            distinguishing_fields.disambiguation,
            distinguishing_fields.abbreviation,
            distinguishing_fields.description,
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
        WordMeaningQuery::exists_by_word_and_meaning_id(
            database_connection,
            slovene_word_id.upcast_to_word_id(),
            slovene_word_meaning_id.upcast_to_word_meaning_id(),
        )
        .await
    }
}
