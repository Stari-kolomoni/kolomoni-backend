use sea_orm_migration::prelude::*;

use crate::m20230624_133941_create_users_table::User;

const FOREIGN_KEY_USER_ID: &str = "fk_user-permission_user-id";
const FOREIGN_KEY_PERMISSION_ID: &str = "fk_user-permission_permission-id";


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
                        ColumnDef::new(Permission::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Permission::Name)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Permission::Description).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Permission::Table)
                    .col(Permission::Name)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(UserPermission::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(UserPermission::UserId).integer().not_null())
                    .col(
                        ColumnDef::new(UserPermission::PermissionId)
                            .integer()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(UserPermission::UserId)
                            .col(UserPermission::PermissionId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(FOREIGN_KEY_USER_ID)
                            .from(UserPermission::Table, UserPermission::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(FOREIGN_KEY_PERMISSION_ID)
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

/// Learn more at <https://docs.rs/sea-query#iden>.
#[derive(Iden)]
pub enum Permission {
    #[iden = "permission"]
    Table,

    #[iden = "id"]
    Id,

    #[iden = "name"]
    Name,

    #[iden = "description"]
    Description,
}

#[derive(Iden)]
pub enum UserPermission {
    #[iden = "user_permission"]
    Table,

    #[iden = "user_id"]
    UserId,

    #[iden = "permission_id"]
    PermissionId,
}
