use sea_orm::entity::prelude::*;


#[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "permission")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    #[sea_orm(unique)]
    pub name: String,

    pub description: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        super::user_permissions::Relation::User.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::user_permissions::Relation::Permission.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
