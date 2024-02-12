use miette::Result;
use miette::{Context, IntoDiagnostic};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use super::super::entities::prelude::WordEnglish;
use crate::entities::word_english;

pub struct EnglishWordQuery;

impl EnglishWordQuery {
    pub async fn word_by_uuid<C: ConnectionTrait>(
        database: &C,
        word_uuid: Uuid,
    ) -> Result<Option<word_english::Model>> {
        WordEnglish::find_by_id(word_uuid)
            .one(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while searching database for english word by UUID.")
    }

    pub async fn word_by_lemma<C: ConnectionTrait>(
        database: &C,
        word_lemma: String,
    ) -> Result<Option<word_english::Model>> {
        WordEnglish::find()
            .filter(word_english::Column::Lemma.eq(word_lemma))
            .one(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while searching database for english word by lemma.")
    }

    pub async fn all_words<C: ConnectionTrait>(database: &C) -> Result<Vec<word_english::Model>> {
        WordEnglish::find()
            .all(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while querying all english words from the database.")
    }
}
