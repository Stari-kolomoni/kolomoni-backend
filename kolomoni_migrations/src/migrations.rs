use kolomoni_migrations_macros::embed_migrations;

embed_migrations!("migrations", "..", "../kolomoni_migrations");
