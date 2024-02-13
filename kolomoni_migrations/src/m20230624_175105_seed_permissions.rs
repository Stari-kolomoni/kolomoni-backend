use sea_orm_migration::prelude::*;

use crate::m20230624_170512_initialize_permission_related_tables::Permission;


/// This is the list of available permissions.
/// 
/// **IMPORTANT: This permission list (or on related migrations) should be kept in sync 
/// with `./kolomoni_auth/src/permissions.rs`.**
/// 
/// We don't keep them in sync automatically because that would mean a migration would
/// not stay the same. We can modify the migration sanely if any only if we're still in 
/// the unstable prototyping phase. Otherwise, opt for a new migration that adds the new permissions.
#[derive(Clone, Copy, Debug)]
pub enum StandardPermission {
    UserSelfRead,
    UserSelfWrite,
    UserAnyRead,
    UserAnyWrite,
    WordCreate,
    WordRead,
    WordUpdate,
    WordDelete,
}

impl StandardPermission {
    pub fn all_permissions() -> Vec<Self> {
        vec![
            Self::UserSelfRead,
            Self::UserSelfWrite,
            Self::UserAnyRead,
            Self::UserAnyWrite,
            Self::WordCreate,
            Self::WordRead,
            Self::WordUpdate,
            Self::WordDelete,
        ]
    }

    pub fn id(&self) -> i32 {
        match self {
            StandardPermission::UserSelfRead => 1,
            StandardPermission::UserSelfWrite => 2,
            StandardPermission::UserAnyRead => 3,
            StandardPermission::UserAnyWrite => 4,
            StandardPermission::WordCreate => 5,
            StandardPermission::WordRead => 6,
            StandardPermission::WordUpdate => 7,
            StandardPermission::WordDelete => 8,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            StandardPermission::UserSelfRead => "user.self:read",
            StandardPermission::UserSelfWrite => "user.self:write",
            StandardPermission::UserAnyRead => "user.any:read",
            StandardPermission::UserAnyWrite => "user.any:write",
            StandardPermission::WordCreate => "word:create",
            StandardPermission::WordRead => "word:read",
            StandardPermission::WordUpdate => "word:update",
            StandardPermission::WordDelete => "word:delete",
        }
    }

    #[rustfmt::skip]
    pub fn description(&self) -> &'static str {
        match self {
            StandardPermission::UserSelfRead => 
                "Allows the user to log in and view their account information.",
            StandardPermission::UserSelfWrite => 
                "Allows the user to update their account information.",
            StandardPermission::UserAnyRead => 
                "Allows the user to view public account information of any other user.",
            StandardPermission::UserAnyWrite => 
                "Allows the user to update account information of any other user.",
            StandardPermission::WordCreate => 
                "Allows the user to create words in the dictionary.",
            StandardPermission::WordRead => 
                "Allows the user to read words in the dictionary.",
            StandardPermission::WordUpdate => 
                "Allows the user to update existing words in the dictionary (but not delete them).",
            StandardPermission::WordDelete => 
                "Allows the user to delete words from the dictionary.",
        }
    }
}


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


#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for permission in StandardPermission::all_permissions() {
            insert_permission(manager, permission.id(), permission.name(), permission.description()).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for permission in StandardPermission::all_permissions() {
            delete_permission_by_id(manager, permission.id()).await?;
        }

        Ok(())
    }
}
