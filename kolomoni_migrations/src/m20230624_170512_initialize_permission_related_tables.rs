use sea_orm_migration::prelude::*;

use crate::m20230624_133941_create_users_table::User;



/// Learn more at <https://docs.rs/sea-query#iden>.
#[derive(DeriveIden)]
pub enum Permission {
    #[sea_orm(iden = "permission")]
    Table,

    #[sea_orm(iden = "id")]
    Id,

    #[sea_orm(iden = "name")]
    Name,

    #[sea_orm(iden = "description")]
    Description,
}

const PERMISSION_PK_CONSTRAINT_NAME: &str = "pk__permission";
const PERMISSION_IDX_ON_ID_INDEX_NAME: &str = "index__permission__on__id";
const PERMISSION_IDX_ON_NAME_INDEX_NAME: &str = "index__permission__on__name";
const PERMISSION_UNIQUE_ON_NAME_CONSTRAINT_NAME: &str = "unique__permission__name";


#[derive(DeriveIden)]
pub enum UserPermission {
    #[sea_orm(iden = "user_permission")]
    Table,

    #[sea_orm(iden = "user_id")]
    UserId,

    #[sea_orm(iden = "permission_id")]
    PermissionId,
}

const USER_PERMISSION_PK_CONSTRAINT_NAME: &str = "pk__user_permission";
const USER_PERMISSION_FK_USER_ID: &str = "fk__user_permission__user_id";
const USER_PERMISSION_FK_PERMISSION_ID: &str = "fk__user_permission__permission_id";



#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Permission::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new_with_type(Permission::Id, ColumnType::Integer)
                            .not_null()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new_with_type(Permission::Name, ColumnType::String(None))
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new_with_type(Permission::Description, ColumnType::String(None))
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .name(PERMISSION_PK_CONSTRAINT_NAME)
                            .table(Permission::Table)
                            .col(Permission::Id)
                            .primary(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(PERMISSION_IDX_ON_ID_INDEX_NAME)
                    .table(Permission::Table)
                    .col(Permission::Id)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(PERMISSION_IDX_ON_NAME_INDEX_NAME)
                    .table(Permission::Table)
                    .col(Permission::Name)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(PERMISSION_UNIQUE_ON_NAME_CONSTRAINT_NAME)
                    .table(Permission::Table)
                    .col(Permission::Name)
                    .unique()
                    .to_owned(),
            )
            .await?;


        manager
            .create_table(
                Table::create()
                    .table(UserPermission::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new_with_type(UserPermission::UserId, ColumnType::Integer)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new_with_type(UserPermission::PermissionId, ColumnType::Integer)
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .name(USER_PERMISSION_PK_CONSTRAINT_NAME)
                            .col(UserPermission::UserId)
                            .col(UserPermission::PermissionId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(USER_PERMISSION_FK_USER_ID)
                            .from(UserPermission::Table, UserPermission::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(USER_PERMISSION_FK_PERMISSION_ID)
                            .from(
                                UserPermission::Table,
                                UserPermission::PermissionId,
                            )
                            .to(Permission::Table, Permission::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserPermission::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Permission::Table).to_owned())
            .await
    }
}
