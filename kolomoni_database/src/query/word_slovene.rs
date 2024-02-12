use miette::{Context, IntoDiagnostic, Result};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use super::super::entities::prelude::WordSlovene;
use crate::entities::word_slovene;

pub struct SloveneWordQuery;

impl SloveneWordQuery {
    pub async fn word_by_uuid<C: ConnectionTrait>(
        database: &C,
        word_uuid: Uuid,
    ) -> Result<Option<word_slovene::Model>> {
        WordSlovene::find_by_id(word_uuid)
            .one(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while searching database for slovene word by UUID.")
    }

    pub async fn word_by_lemma<C: ConnectionTrait>(
        database: &C,
        word_lemma: String,
    ) -> Result<Option<word_slovene::Model>> {
        WordSlovene::find()
            .filter(word_slovene::Column::Lemma.eq(word_lemma))
            .one(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while seaching database for slovene word by lemma.")
    }

    pub async fn all_words<C: ConnectionTrait>(database: &C) -> Result<Vec<word_slovene::Model>> {
        WordSlovene::find()
            .all(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while querying all slovene words from the database.")
    }
}
