use sea_orm_migration::prelude::*;

use crate::m20230624_170512_create_permissions::Permission;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[rustfmt::skip]
const STANDARD_PERMISSIONS: [(&str, &str); 1] = [
    (
        "login",
        "Allows the user to log into their account.",
    ),
    // TODO Add other permissions.
];

async fn insert_permission<'manager>(
    manager: &SchemaManager<'manager>,
    name: &str,
    description: &str,
) -> Result<(), DbErr> {
    let insert = Query::insert()
        .into_table(Permission::Table)
        .columns([Permission::Name, Permission::Description])
        .values_panic([name.into(), description.into()])
        .to_owned();

    manager.exec_stmt(insert).await
}

async fn delete_permission_by_name<'manager>(
    manager: &SchemaManager<'manager>,
    name: &str,
) -> Result<(), DbErr> {
    let delete = Query::delete()
        .from_table(Permission::Table)
        .cond_where(Expr::col(Permission::Name).eq(name))
        .to_owned();

    manager.exec_stmt(delete).await
}


#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for (name, description) in STANDARD_PERMISSIONS {
            insert_permission(manager, name, description).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for (name, _) in STANDARD_PERMISSIONS {
            delete_permission_by_name(manager, name).await?;
        }

        todo!();
    }
}
