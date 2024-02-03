//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.12

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "permission")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub name: String,
    pub description: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::user_permission::Entity")]
    UserPermission,
}

impl Related<super::user_permission::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserPermission.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        super::user_permission::Relation::User.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::user_permission::Relation::Permission.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
