use sea_orm_migration::prelude::*;

const USERNAME_INDEX_NAME: &str = "idx-username";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(User::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(User::Username).string().not_null())
                    .col(ColumnDef::new(User::DisplayName).string().not_null())
                    .col(ColumnDef::new(User::HashedPassword).string().not_null())
                    .col(
                        ColumnDef::new(User::JoinedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(User::LastModifiedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(User::LastActiveAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(USERNAME_INDEX_NAME)
                    .table(User::Table)
                    .col(User::Username)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name(USERNAME_INDEX_NAME).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum User {
    #[iden = "user"]
    Table,

    #[iden = "id"]
    Id,

    #[iden = "username"]
    Username,

    #[iden = "display_name"]
    DisplayName,

    #[iden = "hashed_password"]
    HashedPassword,

    #[iden = "joined_at"]
    JoinedAt,

    #[iden = "last_modified_at"]
    LastModifiedAt,

    #[iden = "last_active_at"]
    LastActiveAt,
}
