use sea_orm_migration::prelude::*;

use crate::{
    m20230624_133941_create_users_table::User,
    m20230624_170512_initialize_permission_related_tables::Permission,
};



#[derive(DeriveIden)]
pub enum Role {
    #[sea_orm(iden = "role")]
    Table,

    #[sea_orm(iden = "id")]
    Id,

    #[sea_orm(iden = "name")]
    Name,

    #[sea_orm(iden = "description")]
    Description,
}

const ROLE_PK_CONSTRAINT_NAME: &str = "pk__role";
const ROLE_UNIQUE_ON_NAME_CONSTRAINT_NAME: &str = "unique__role__name";



#[derive(DeriveIden)]
pub enum RolePermission {
    #[sea_orm(iden = "role_permission")]
    Table,

    #[sea_orm(iden = "role_id")]
    RoleId,

    #[sea_orm(iden = "permission_id")]
    PermissionId,
}

const ROLE_PERMISSION_PK_CONSTRAINT_NAME: &str = "pk__role_permission";
const ROLE_PERMISSION_FK_CONSTRAINT_NAME_ROLE: &str = "fk__role_permission__role_id__role";
const ROLE_PERMISSION_FK_CONSTRAINT_NAME_PERMISSION: &str =
    "fk__role_permission__permission_id__permission";


#[derive(DeriveIden)]
pub enum UserRole {
    #[sea_orm(iden = "user_role")]
    Table,

    #[sea_orm(iden = "user_id")]
    UserId,

    #[sea_orm(iden = "role_id")]
    RoleId,
}

const USER_ROLE_PK_CONSTRAINT_NAME: &str = "pk__user_role";
const USER_ROLE_FK_CONSTRAINT_NAME_USER: &str = "fk__user_role__user_id__user";
const USER_ROLE_FK_CONSTRAINT_NAME_ROLE: &str = "fk__user_role__role_id__role";



#[derive(DeriveMigrationName)]
pub struct Migration;


#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Role::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new_with_type(Role::Id, ColumnType::Integer)
                            .not_null()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new_with_type(Role::Name, ColumnType::String(None)).not_null())
                    .col(
                        ColumnDef::new_with_type(Role::Description, ColumnType::String(None))
                            .not_null(),
                    )
                    .primary_key(Index::create().name(ROLE_PK_CONSTRAINT_NAME).col(Role::Id))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(ROLE_UNIQUE_ON_NAME_CONSTRAINT_NAME)
                    .table(Role::Table)
                    .col(Role::Name)
                    .unique()
                    .to_owned(),
            )
            .await?;


        manager
            .create_table(
                Table::create()
                    .table(RolePermission::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new_with_type(RolePermission::RoleId, ColumnType::Integer)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new_with_type(RolePermission::PermissionId, ColumnType::Integer)
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .name(ROLE_PERMISSION_PK_CONSTRAINT_NAME)
                            .col(RolePermission::PermissionId)
                            .col(RolePermission::RoleId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(ROLE_PERMISSION_FK_CONSTRAINT_NAME_ROLE)
                            .from(RolePermission::Table, RolePermission::RoleId)
                            .to(Role::Table, Role::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(ROLE_PERMISSION_FK_CONSTRAINT_NAME_PERMISSION)
                            .from(
                                RolePermission::Table,
                                RolePermission::PermissionId,
                            )
                            .to(Permission::Table, Permission::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;


        manager
            .create_table(
                Table::create()
                    .table(UserRole::Table)
                    .if_not_exists()
                    .col(ColumnDef::new_with_type(UserRole::UserId, ColumnType::Integer).not_null())
                    .col(ColumnDef::new_with_type(UserRole::RoleId, ColumnType::Integer).not_null())
                    .primary_key(
                        Index::create()
                            .name(USER_ROLE_PK_CONSTRAINT_NAME)
                            .col(UserRole::UserId)
                            .col(UserRole::RoleId)
                            .primary(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(USER_ROLE_FK_CONSTRAINT_NAME_USER)
                            .from(UserRole::Table, UserRole::UserId)
                            .to(User::Table, User::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(USER_ROLE_FK_CONSTRAINT_NAME_ROLE)
                            .from(UserRole::Table, UserRole::RoleId)
                            .to(Role::Table, Role::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserRole::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(RolePermission::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Role::Table).to_owned())
            .await
    }
}
