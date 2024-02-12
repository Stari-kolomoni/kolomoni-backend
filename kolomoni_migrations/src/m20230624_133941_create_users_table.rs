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

const USER_PK_CONSTRAINT_NAME: &str = "pk__user";
const USER_UNIQUE_ON_USERNAME_CONSTRAINT_NAME: &str = "unique__user__username";
const USER_UNIQUE_ON_DISPLAY_NAME_CONSTRAINT_NAME: &str = "unique__user__display_name";
const USER_IDX_ON_ID_INDEX_NAME: &str = "index__user__on__id";
const USER_IDX_ON_USERNAME_INDEX_NAME: &str = "index__user__on__username";




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
                        ColumnDef::new_with_type(User::Id, ColumnType::Integer)
                            .not_null()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new_with_type(User::Username, ColumnType::String(None))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new_with_type(User::DisplayName, ColumnType::String(None))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new_with_type(User::HashedPassword, ColumnType::String(None))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new_with_type(User::JoinedAt, ColumnType::TimestampWithTimeZone)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new_with_type(
                            User::LastModifiedAt,
                            ColumnType::TimestampWithTimeZone,
                        )
                        .not_null(),
                    )
                    .col(
                        ColumnDef::new_with_type(
                            User::LastActiveAt,
                            ColumnType::TimestampWithTimeZone,
                        )
                        .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .name(USER_PK_CONSTRAINT_NAME)
                            .table(User::Table)
                            .col(User::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(USER_IDX_ON_ID_INDEX_NAME)
                    .table(User::Table)
                    .col(User::Id)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(USER_IDX_ON_USERNAME_INDEX_NAME)
                    .table(User::Table)
                    .col(User::Username)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(USER_UNIQUE_ON_USERNAME_CONSTRAINT_NAME)
                    .table(User::Table)
                    .col(User::Username)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(USER_UNIQUE_ON_DISPLAY_NAME_CONSTRAINT_NAME)
                    .table(User::Table)
                    .col(User::DisplayName)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}
