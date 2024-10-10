use chrono::Utc;
use kolomoni_core::ids::{SloveneWordId, SloveneWordMeaningId};
use sqlx::{PgConnection, Postgres, QueryBuilder};

use super::SloveneWordMeaningModel;
use crate::{entities::InternalSloveneWordMeaningModel, IntoExternalModel, QueryError, QueryResult};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NewSloveneWordMeaning {
    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,
}


#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SloveneWordMeaningUpdate {
    pub disambiguation: Option<Option<String>>,

    pub abbreviation: Option<Option<String>>,

    pub description: Option<Option<String>>,
}

impl SloveneWordMeaningUpdate {
    fn no_values_to_update(&self) -> bool {
        self.disambiguation.is_none() && self.abbreviation.is_none() && self.description.is_none()
    }
}


fn build_slovene_word_meaning_update_query(
    slovene_word_meaning_id: SloveneWordMeaningId,
    values_to_update: SloveneWordMeaningUpdate,
) -> QueryBuilder<'static, Postgres> {
    let mut update_query_builder: QueryBuilder<Postgres> =
        QueryBuilder::new("UPDATE kolomoni.word_slovene_meaning SET ");

    let mut separated_set_expressions = update_query_builder.separated(", ");


    if let Some(new_disambiguation) = values_to_update.disambiguation {
        separated_set_expressions.push_unseparated("disambiguation = ");
        separated_set_expressions.push_bind(new_disambiguation);
    }

    if let Some(new_abbreviation) = values_to_update.abbreviation {
        separated_set_expressions.push_unseparated("abbreviation = ");
        separated_set_expressions.push_bind(new_abbreviation);
    }

    if let Some(new_description) = values_to_update.description {
        separated_set_expressions.push_unseparated("description = ");
        separated_set_expressions.push_bind(new_description);
    }


    update_query_builder.push(" WHERE word_meaning_id = ");
    update_query_builder.push_bind(slovene_word_meaning_id.into_uuid());

    update_query_builder
}




pub struct SloveneWordMeaningMutation;

impl SloveneWordMeaningMutation {
    pub async fn create(
        database_connection: &mut PgConnection,
        slovene_word_id: SloveneWordId,
        meaning_to_create: NewSloveneWordMeaning,
    ) -> QueryResult<SloveneWordMeaningModel> {
        let new_meaning_id = SloveneWordMeaningId::generate();
        let new_meaning_created_at = Utc::now();
        let new_meaning_last_modified_at = new_meaning_created_at;


        let internal_meaning_query_result = sqlx::query!(
            "INSERT INTO kolomoni.word_meaning (id, word_id) \
                VALUES ($1, $2)",
            new_meaning_id.into_uuid(),
            slovene_word_id.into_uuid()
        )
        .execute(&mut *database_connection)
        .await?;

        if internal_meaning_query_result.rows_affected() != 1 {
            return Err(QueryError::database_inconsistency(format!(
                "inserted word meaning, but got abnormal number of affected rows ({})",
                internal_meaning_query_result.rows_affected()
            )));
        }


        let internal_slovene_meaning = sqlx::query_as!(
            InternalSloveneWordMeaningModel,
            "INSERT INTO kolomoni.word_slovene_meaning \
                (word_meaning_id, disambiguation, abbreviation, \
                description, created_at, last_modified_at) \
                VALUES ($1, $2, $3, $4, $5, $6) \
                RETURNING \
                    word_meaning_id, disambiguation, abbreviation, \
                    description, created_at, last_modified_at",
            new_meaning_id.into_uuid(),
            meaning_to_create.disambiguation,
            meaning_to_create.abbreviation,
            meaning_to_create.description,
            new_meaning_created_at,
            new_meaning_last_modified_at
        )
        .fetch_one(database_connection)
        .await?;


        Ok(internal_slovene_meaning.into_external_model())
    }

    pub async fn update(
        database_connection: &mut PgConnection,
        slovene_word_meaning_id: SloveneWordMeaningId,
        values_to_update: SloveneWordMeaningUpdate,
    ) -> QueryResult<bool> {
        if values_to_update.no_values_to_update() {
            return Ok(true);
        }


        let mut update_query_builder =
            build_slovene_word_meaning_update_query(slovene_word_meaning_id, values_to_update);

        let query_result = update_query_builder
            .build()
            .execute(database_connection)
            .await?;


        Ok(query_result.rows_affected() == 1)
    }

    pub async fn delete(
        database_connection: &mut PgConnection,
        slovene_word_meaning_id: SloveneWordMeaningId,
    ) -> QueryResult<bool> {
        let query_result = sqlx::query!(
            "DELETE FROM kolomoni.word_slovene_meaning \
                WHERE word_meaning_id = $1",
            slovene_word_meaning_id.into_uuid()
        )
        .execute(database_connection)
        .await?;

        if query_result.rows_affected() > 1 {
            return Err(QueryError::database_inconsistency(format!(
                "while deleting slovene word meaning {} more than one row was affected ({})",
                slovene_word_meaning_id,
                query_result.rows_affected()
            )));
        }

        Ok(query_result.rows_affected() == 1)
    }
}
