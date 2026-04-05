use kolomoni_core::ids::{EnglishWordId, EnglishWordMeaningId};
use sqlx::PgConnection;

use super::EnglishWordMeaningModelWithDetails;
use crate::{
    entities::{
        word_meaning::WordMeaningQuery,
        word_meaning_english::internal_weak::WeakInternalEnglishWordMeaningModelWithDetails,
    },
    IntoExternalModel,
    QueryError,
    QueryResult,
    TryIntoStronglyTypedInternalModel,
};


pub struct EnglishWordMeaningLookup {
    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,
}


pub struct EnglishWordMeaningQuery;

impl EnglishWordMeaningQuery {
    pub async fn get_all_by_english_word_id(
        database_connection: &mut PgConnection,
        english_word_id: EnglishWordId,
    ) -> QueryResult<Vec<EnglishWordMeaningModelWithDetails>> {
        let weak_internal_meanings = sqlx::query_file_as!(
            WeakInternalEnglishWordMeaningModelWithDetails,
            "src/entities/word_meaning_english/queries/all_meanings_for_given_english_word.sql",
            english_word_id.into_uuid()
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
        english_word_id: EnglishWordId,
        english_word_meaning_id: EnglishWordMeaningId,
    ) -> QueryResult<Option<EnglishWordMeaningModelWithDetails>> {
        let weak_internal_meaning = sqlx::query_file_as!(
            WeakInternalEnglishWordMeaningModelWithDetails,
            "src/entities/word_meaning_english/queries/single_meaning_by_both_ids.sql",
            english_word_id.into_uuid(),
            english_word_meaning_id.into_uuid()
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
        english_word_meaning_id: EnglishWordMeaningId,
    ) -> QueryResult<bool> {
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS (\
                SELECT 1 \
                    FROM kolomoni.word_meaning_english wme \
                    WHERE wme.word_meaning_id = $1
            )",
            english_word_meaning_id.into_uuid()
        )
        .fetch_one(database_connection)
        .await?;

        Ok(exists.unwrap_or(false))
    }

    pub async fn exists_by_distinguishing_fields(
        database_connection: &mut PgConnection,
        distinguishing_fields: EnglishWordMeaningLookup,
    ) -> QueryResult<bool> {
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS (\
                SELECT 1 \
                    FROM kolomoni.word_meaning_english wme \
                    WHERE wme.disambiguation = $1 \
                        AND wme.abbreviation = $2 \
                        AND wme.description = $3
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
        english_word_id: EnglishWordId,
        english_word_meaning_id: EnglishWordMeaningId,
    ) -> QueryResult<bool> {
        WordMeaningQuery::exists_by_word_and_meaning_id(
            database_connection,
            english_word_id.upcast_to_word_id(),
            english_word_meaning_id.upcast_to_word_meaning_id(),
        )
        .await
    }
}
