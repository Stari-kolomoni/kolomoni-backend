use sea_orm_migration::prelude::*;


/// Learn more at <https://docs.rs/sea-query#iden>.
#[derive(DeriveIden)]
pub enum User {
    #[sea_orm(iden = "user")]
    Table,

    #[sea_orm(iden = "id")]
    Id,

    #[sea_orm(iden = "username")]
    Username,

    #[sea_orm(iden = "display_name")]
    DisplayName,

    #[sea_orm(iden = "hashed_password")]
    HashedPassword,

    #[sea_orm(iden = "joined_at")]
    JoinedAt,

    #[sea_orm(iden = "last_modified_at")]
    LastModifiedAt,

    #[sea_orm(iden = "last_active_at")]
    LastActiveAt,
}

const USER_TABLE_INDEX_ON_USERNAME: &str = "index__user__on__username";




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
                    .col(
                        ColumnDef::new(User::Username)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(User::DisplayName)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
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
                    .name(USER_TABLE_INDEX_ON_USERNAME)
                    .table(User::Table)
                    .col(User::Username)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name(USER_TABLE_INDEX_ON_USERNAME).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}
