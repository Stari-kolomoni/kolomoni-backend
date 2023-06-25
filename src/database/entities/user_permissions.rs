use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "user_permission")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub user_id: i32,

    #[sea_orm(primary_key)]
    pub permission_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id"
    )]
    User,

    #[sea_orm(
        belongs_to = "super::permissions::Entity",
        from = "Column::PermissionId",
        to = "super::permissions::Column::Id"
    )]
    Permission,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::permissions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Permission.def()
    }
}


impl ActiveModelBehavior for ActiveModel {}
