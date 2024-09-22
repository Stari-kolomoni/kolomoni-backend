use futures_core::stream::BoxStream;
use kolomoni_core::id::CategoryId;
use sqlx::PgConnection;

use super::CategoryModel;
use crate::{IntoExternalModel, QueryError, QueryResult};

type RawCategoryStream<'c> = BoxStream<'c, Result<super::InternalCategoryModel, sqlx::Error>>;

create_async_stream_wrapper!(
    pub struct CategoryStream<'c>;
    transforms stream RawCategoryStream<'c> => stream of QueryResult<super::CategoryModel>:
        |value|
            value.map(
                |some| some
                    .map(super::InternalCategoryModel::into_external_model)
                    .map_err(|error| QueryError::SqlxError { error })
            )
);


pub struct CategoryQuery;

impl CategoryQuery {
    pub async fn get_all_categories(database_connection: &mut PgConnection) -> CategoryStream<'_> {
        let internal_category_stream = sqlx::query_as!(
            super::InternalCategoryModel,
            "SELECT \
                    id, parent_category_id, name_sl, name_en, \
                    created_at, last_modified_at \
                FROM kolomoni.category",
        )
        .fetch(database_connection);

        CategoryStream::new(internal_category_stream)
    }

    pub async fn get_by_id(
        database_connection: &mut PgConnection,
        category_id: CategoryId,
    ) -> QueryResult<Option<CategoryModel>> {
        let internal_category = sqlx::query_as!(
            super::InternalCategoryModel,
            "SELECT \
                    id, parent_category_id, name_sl, name_en, \
                    created_at, last_modified_at \
                FROM kolomoni.category \
                WHERE id = $1",
            category_id.into_uuid()
        )
        .fetch_optional(database_connection)
        .await?;

        Ok(internal_category.map(|category| category.into_external_model()))
    }

    pub async fn exists_by_id(
        database_connection: &mut PgConnection,
        category_id: CategoryId,
    ) -> QueryResult<bool> {
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS (\
                SELECT 1 \
                    FROM kolomoni.category \
                    WHERE id = $1
            )",
            category_id.into_uuid()
        )
        .fetch_one(database_connection)
        .await?;

        Ok(exists.unwrap_or(false))
    }

    pub async fn exists_by_slovene_name(
        database_connection: &mut PgConnection,
        slovene_category_name: &str,
    ) -> QueryResult<bool> {
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS (\
                SELECT 1 \
                    FROM kolomoni.category \
                    WHERE name_sl = $1
            )",
            slovene_category_name
        )
        .fetch_one(database_connection)
        .await?;

        Ok(exists.unwrap_or(false))
    }

    pub async fn exists_by_english_name(
        database_connection: &mut PgConnection,
        english_category_name: &str,
    ) -> QueryResult<bool> {
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS (\
                SELECT 1 \
                    FROM kolomoni.category \
                    WHERE name_en = $1
            )",
            english_category_name
        )
        .fetch_one(database_connection)
        .await?;

        Ok(exists.unwrap_or(false))
    }
}
