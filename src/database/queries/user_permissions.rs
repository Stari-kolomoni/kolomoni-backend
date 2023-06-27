use anyhow::{anyhow, Result};
use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter};

use crate::database::entities;

#[allow(dead_code)]
pub struct Query {}

impl Query {
    pub async fn get_user_permission_names_by_username(
        database: &DbConn,
        username: &str,
    ) -> Result<Option<Vec<String>>> {
        let results = entities::users::Entity::find()
            .filter(entities::users::Column::Username.eq(username))
            .find_with_related(entities::permissions::Entity)
            .all(database)
            .await?;

        if results.is_empty() {
            return Ok(None);
        } else if results.len() != 1 {
            return Err(anyhow!(
                "BUG: more than one result for user by username!",
            ));
        }

        let (_, permissions) = &results[0];

        Ok(Some(
            permissions
                .iter()
                .map(|permission| permission.name.clone())
                .collect(),
        ))
    }

    pub async fn get_user_permission_names_by_user_id(
        database: &DbConn,
        user_id: i32,
    ) -> Result<Option<Vec<String>>> {
        let results = entities::users::Entity::find_by_id(user_id)
            .find_with_related(entities::permissions::Entity)
            .all(database)
            .await?;

        if results.is_empty() {
            return Ok(None);
        } else if results.len() != 1 {
            return Err(anyhow!(
                "BUG: more than one result for user by ID!"
            ));
        }

        let (_, permissions) = &results[0];

        Ok(Some(
            permissions
                .iter()
                .map(|permission| permission.name.clone())
                .collect(),
        ))
    }
}
