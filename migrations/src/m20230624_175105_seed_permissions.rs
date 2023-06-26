use sea_orm_migration::prelude::*;

use crate::m20230624_170512_create_permissions_table::Permission;

#[derive(DeriveMigrationName)]
pub struct Migration;

// See `auth.rs` for the source of these permissions.
#[rustfmt::skip]
const STANDARD_PERMISSIONS: [(i32, &str, &str); 4] = [
    (
        1,
        "user.self:read",
        "Allows the user to log in and view their account information.",
    ),
    (
        2,
        "user.self:write",
        "Allows the user to update their account information."
    ),
    (
        3,
        "user.any:read",
        "Allows the user to view public account information of any other user."
    ),
    (
        4,
        "user.any:write",
        "Allows the user to update public account information of any other user."
    ),
    // TODO Add other permissions.
];

async fn insert_permission<'manager>(
    manager: &SchemaManager<'manager>,
    id: i32,
    name: &str,
    description: &str,
) -> Result<(), DbErr> {
    let insert = Query::insert()
        .into_table(Permission::Table)
        .columns([Permission::Id, Permission::Name, Permission::Description])
        .values_panic([id.into(), name.into(), description.into()])
        .to_owned();

    manager.exec_stmt(insert).await
}

async fn delete_permission_by_id<'manager>(
    manager: &SchemaManager<'manager>,
    id: i32,
) -> Result<(), DbErr> {
    let delete = Query::delete()
        .from_table(Permission::Table)
        .cond_where(Expr::col(Permission::Id).eq(id))
        .to_owned();

    manager.exec_stmt(delete).await
}


#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for (id, name, description) in STANDARD_PERMISSIONS {
            insert_permission(manager, id, name, description).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for (id, _, _) in STANDARD_PERMISSIONS {
            delete_permission_by_id(manager, id).await?;
        }

        todo!();
    }
}
