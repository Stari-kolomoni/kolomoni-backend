#![allow(unreachable_code, unused_variables)]

pub use sea_orm_migration::prelude::*;

mod m20230624_133941_create_users_table;
mod m20230624_170512_create_permissions_table;
mod m20230624_175105_seed_permissions;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230624_133941_create_users_table::Migration),
            Box::new(m20230624_170512_create_permissions_table::Migration),
            Box::new(m20230624_175105_seed_permissions::Migration),
        ]
    }
}
