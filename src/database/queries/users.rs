use anyhow::{Context, Result};
use sea_orm::{DbConn, EntityTrait};

use super::super::entities::{users, users::Entity as User};

#[allow(dead_code)]
pub struct Query {}

impl Query {
    pub async fn get_user_by_id(database: &DbConn, id: i32) -> Result<Option<users::Model>> {
        User::find_by_id(id)
            .one(database)
            .await
            .with_context(|| "Failed to search database for user.")
    }
}
