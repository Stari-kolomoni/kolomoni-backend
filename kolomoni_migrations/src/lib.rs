pub use sea_orm_migration::prelude::*;

mod m20230624_133941_create_users_table;
mod m20230624_170512_create_permissions_table;
mod m20230624_175105_seed_permissions;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        // Do not touch the order of this vector unless you know what you are doing.
        // This will be automatically extended with the new migration after you run
        // `sea-orm-cli migrate generate ...`.
        vec![
            Box::new(m20230624_133941_create_users_table::Migration),
            Box::new(m20230624_170512_create_permissions_table::Migration),
            Box::new(m20230624_175105_seed_permissions::Migration),
        ]
    }
}
