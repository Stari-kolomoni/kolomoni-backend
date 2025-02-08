use chrono::Utc;
use kolomoni_core::ids::{EnglishWordId, EnglishWordMeaningId};
use sqlx::{PgConnection, Postgres, QueryBuilder};

use super::internal::InternalEnglishWordMeaningModel;
use super::EnglishWordMeaningModel;
use crate::entities::word_meaning::NewWordMeaning;
use crate::entities::word_meaning::WordMeaningMutation;
use crate::entities::word_meaning_english::internal_insert_only::InsertOnlyInternalEnglishWordMeaningModel;
use crate::{IntoExternalModel, QueryError, QueryResult};


#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NewEnglishWordMeaning {
    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,
}


#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EnglishWordMeaningUpdate {
    pub disambiguation: Option<Option<String>>,

    pub abbreviation: Option<Option<String>>,

    pub description: Option<Option<String>>,
}

impl EnglishWordMeaningUpdate {
    fn no_values_to_update(&self) -> bool {
        self.disambiguation.is_none() && self.abbreviation.is_none() && self.description.is_none()
    }
}


fn build_english_word_meaning_update_query(
    english_word_meaning_id: EnglishWordMeaningId,
    values_to_update: EnglishWordMeaningUpdate,
) -> QueryBuilder<'static, Postgres> {
    let mut update_query_builder: QueryBuilder<Postgres> =
        QueryBuilder::new("UPDATE kolomoni.word_meaning_english SET ");

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
    update_query_builder.push_bind(english_word_meaning_id.into_uuid());

    update_query_builder
}



pub struct EnglishWordMeaningMutation;

impl EnglishWordMeaningMutation {
    pub async fn create(
        database_connection: &mut PgConnection,
        english_word_id: EnglishWordId,
        meaning_to_create: NewEnglishWordMeaning,
    ) -> QueryResult<EnglishWordMeaningModel> {
        let new_meaning_created_at = Utc::now();
        let new_meaning_last_modified_at = new_meaning_created_at;


        let new_word_meaning = WordMeaningMutation::create(
            database_connection,
            NewWordMeaning {
                word_id: english_word_id.to_word_id(),
                created_at: new_meaning_created_at,
                last_modified_at: new_meaning_last_modified_at,
            },
        )
        .await?;


        let new_word_meaning_english = sqlx::query_as!(
            InsertOnlyInternalEnglishWordMeaningModel,
            "INSERT INTO kolomoni.word_meaning_english \
                    (word_meaning_id, disambiguation, abbreviation, description) \
                VALUES \
                    ($1, $2, $3, $4) \
                RETURNING \
                    word_meaning_id, disambiguation, abbreviation, description",
            new_word_meaning.word_meaning_id.into_uuid(),
            meaning_to_create.disambiguation,
            meaning_to_create.abbreviation,
            meaning_to_create.description,
        )
        .fetch_one(database_connection)
        .await?;


        let complete_internal_meaning_model = InternalEnglishWordMeaningModel {
            word_id: new_word_meaning.word_id.into_uuid(),
            word_meaning_id: new_word_meaning.word_meaning_id.into_uuid(),
            created_at: new_word_meaning.created_at,
            last_modified_at: new_word_meaning.last_modified_at,
            disambiguation: new_word_meaning_english.disambiguation,
            abbreviation: new_word_meaning_english.abbreviation,
            description: new_word_meaning_english.description,
        };

        Ok(complete_internal_meaning_model.into_external_model())
    }

    pub async fn update(
        database_connection: &mut PgConnection,
        english_word_meaning_id: EnglishWordMeaningId,
        values_to_update: EnglishWordMeaningUpdate,
    ) -> QueryResult<bool> {
        if values_to_update.no_values_to_update() {
            return Ok(true);
        };

        let mut update_query_builder =
            build_english_word_meaning_update_query(english_word_meaning_id, values_to_update);

        let query_result = update_query_builder
            .build()
            .execute(database_connection)
            .await?;

        Ok(query_result.rows_affected() == 1)
    }

    pub async fn delete(
        database_connection: &mut PgConnection,
        english_word_meaning_id: EnglishWordMeaningId,
    ) -> QueryResult<bool> {
        let query_result = sqlx::query!(
            "DELETE FROM kolomoni.word_meaning_english wme \
                WHERE wme.word_meaning_id = $1",
            english_word_meaning_id.into_uuid()
        )
        .execute(database_connection)
        .await?;

        if query_result.rows_affected() > 1 {
            return Err(QueryError::database_inconsistency(format!(
                "while deleting an english word meaning {}, more than one row was affected ({} rows)!",
                english_word_meaning_id,
                query_result.rows_affected()
            )));
        }

        Ok(query_result.rows_affected() == 1)
    }
}



#[cfg(test)]
mod test {
    use sqlx::Execute;
    use uuid::Uuid;

    use super::*;

    #[test]
    fn builds_correct_meaning_update_queries() {
        let meaning_id = EnglishWordMeaningId::new(Uuid::nil());

        assert_eq!(
            build_english_word_meaning_update_query(
                meaning_id,
                EnglishWordMeaningUpdate {
                    abbreviation: Some(Some("a".into())),
                    description: Some(None),
                    disambiguation: Some(None),
                }
            )
                .build()
                .sql(),
            format!(
                "UPDATE kolomoni.word_meaning_english SET abbreviation = $1 WHERE word_meaning_id = {}",
                meaning_id.into_uuid()
            )
        );

        // TODO Other tests.
    }
}
