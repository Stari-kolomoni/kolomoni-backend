pub use sea_orm_migration::prelude::*;

mod m20230624_133941_create_users_table;
mod m20230624_170512_initialize_permission_related_tables;
mod m20230624_175105_seed_permissions;
mod m20230624_177000_initialize_role_related_tables;
mod m20230624_177050_seed_roles;
mod m20240206_234618_create_word_tables;
mod m20240219_161147_create_word_suggestion_and_translation_tables;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        // Do not touch the order of this vector unless you know what you are doing.
        // This will be automatically extended with the new migration after you run
        // `sea-orm-cli migrate generate ...`.
        vec![
            Box::new(m20230624_133941_create_users_table::Migration),
            Box::new(m20230624_170512_initialize_permission_related_tables::Migration),
            Box::new(m20230624_175105_seed_permissions::Migration),
            Box::new(m20230624_177000_initialize_role_related_tables::Migration),
            Box::new(m20230624_177050_seed_roles::Migration),
            Box::new(m20240206_234618_create_word_tables::Migration),
            Box::new(m20240219_161147_create_word_suggestion_and_translation_tables::Migration),
        ]
    }
}
