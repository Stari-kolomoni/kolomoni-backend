use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    #[sea_orm(unique, indexed)]
    pub username: String,

    #[sea_orm(unique)]
    pub display_name: String,

    pub hashed_password: String,

    pub joined_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,

    pub last_active_at: DateTime<Utc>,
    // TODO Roles.
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
