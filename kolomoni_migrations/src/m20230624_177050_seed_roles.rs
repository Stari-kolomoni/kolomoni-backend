use sea_orm_migration::prelude::*;

use crate::{
    m20230624_175105_seed_permissions::StandardPermission,
    m20230624_177000_initialize_role_related_tables::{Role, RolePermission},
};

/// This is the list of available roles.
///
/// **IMPORTANT: This role list (or roles on related migrations) should be kept in sync
/// with `./kolomoni_auth/src/roles.rs`.**
///
/// We don't keep them in sync automatically because that would mean a migration would
/// not stay the same. We can modify the migration sanely if any only if we're still in
/// the unstable prototyping phase. Otherwise, opt for a new migration that adds the new permissions.
#[derive(Clone, Copy, Debug)]
pub enum StandardRole {
    User,
    Administrator,
}

impl StandardRole {
    pub fn all_roles() -> Vec<Self> {
        vec![Self::User, Self::Administrator]
    }

    pub fn id(&self) -> i32 {
        match self {
            StandardRole::User => 1,
            StandardRole::Administrator => 2,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            StandardRole::User => "user",
            StandardRole::Administrator => "administrator",
        }
    }

    #[rustfmt::skip]
    pub fn description(&self) -> &'static str {
        match self {
            StandardRole::User =>
                "Normal user with most read permissions.",
            StandardRole::Administrator =>
                "Administrator with almost all permission, including deletions.",
        }
    }

    pub fn permission_list(&self) -> Vec<StandardPermission> {
        match self {
            StandardRole::User => vec![
                StandardPermission::UserSelfRead,
                StandardPermission::UserSelfWrite,
                StandardPermission::UserAnyRead,
                StandardPermission::WordRead,
            ],
            StandardRole::Administrator => vec![
                StandardPermission::UserAnyWrite,
                StandardPermission::WordCreate,
                StandardPermission::WordUpdate,
                StandardPermission::WordDelete,
            ],
        }
    }
}




async fn insert_role<'manager>(
    manager: &SchemaManager<'manager>,
    id: i32,
    name: &str,
    description: &str,
    corresponding_permissions: &[StandardPermission],
) -> Result<(), DbErr> {
    let insert = Query::insert()
        .into_table(Role::Table)
        .columns([Role::Id, Role::Name, Role::Description])
        .values_panic([id.into(), name.into(), description.into()])
        .to_owned();

    manager.exec_stmt(insert).await?;

    for permission in corresponding_permissions {
        let add_permission_to_role_stmt = Query::insert()
            .into_table(RolePermission::Table)
            .columns([RolePermission::RoleId, RolePermission::PermissionId])
            .values_panic([id.into(), permission.id().into()])
            .to_owned();

        manager.exec_stmt(add_permission_to_role_stmt).await?;
    }

    Ok(())
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
        for role in StandardRole::all_roles() {
            insert_role(
                manager,
                role.id(),
                role.name(),
                role.description(),
                &role.permission_list(),
            )
            .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for role in StandardRole::all_roles() {
            delete_role_by_id(manager, role.id()).await?;
        }

        Ok(())
    }
}
