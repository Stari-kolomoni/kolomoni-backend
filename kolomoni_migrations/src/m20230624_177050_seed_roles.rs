use sea_orm_migration::prelude::*;

use crate::m20230624_177000_initialize_role_related_tables::Role;



/// This is the list of available roles.
/// 
/// **IMPORTANT: This role list (or roles on related migrations) should be kept in sync 
/// with `./kolomoni_auth/src/roles.rs`.**
/// 
/// We don't keep them in sync automatically because that would mean a migration would
/// not stay the same. We can modify the migration sanely if any only if we're still in 
/// the unstable prototyping phase. Otherwise, opt for a new migration that adds the new permissions.
#[rustfmt::skip]
const STANDARD_ROLES: [(i32, &str, &str); 2] = [
    (
        1,
        "user",
        "Normal user with most read permissions.",
    ),
    (
        2,
        "administrator",
        "Administrator with almost all permission, including deletions."
    )
];


async fn insert_role<'manager>(
    manager: &SchemaManager<'manager>,
    id: i32,
    name: &str,
    description: &str,
) -> Result<(), DbErr> {
    let insert = Query::insert()
        .into_table(Role::Table)
        .columns([Role::Id, Role::Name, Role::Description])
        .values_panic([id.into(), name.into(), description.into()])
        .to_owned();

    manager.exec_stmt(insert).await
}

async fn delete_role_by_id<'manager>(
    manager: &SchemaManager<'manager>,
    id: i32,
) -> Result<(), DbErr> {
    let delete = Query::delete()
        .from_table(Role::Table)
        .cond_where(Expr::col(Role::Id).eq(id))
        .to_owned();

    manager.exec_stmt(delete).await
}


#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for (id, name, description) in STANDARD_ROLES {
            insert_role(manager, id, name, description).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for (id, _, _) in STANDARD_ROLES {
            delete_role_by_id(manager, id).await?;
        }

        Ok(())
    }
}
