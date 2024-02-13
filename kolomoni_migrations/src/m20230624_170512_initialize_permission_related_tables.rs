use sea_orm_migration::prelude::*;



/// Learn more at <https://docs.rs/sea-query#iden>.
#[derive(DeriveIden)]
pub enum Permission {
    #[sea_orm(iden = "permission")]
    Table,

    #[sea_orm(iden = "id")]
    Id,

    #[sea_orm(iden = "name")]
    Name,

    #[sea_orm(iden = "description")]
    Description,
}

const PERMISSION_PK_CONSTRAINT_NAME: &str = "pk__permission";
const PERMISSION_IDX_ON_ID_INDEX_NAME: &str = "index__permission__on__id";
const PERMISSION_IDX_ON_NAME_INDEX_NAME: &str = "index__permission__on__name";
const PERMISSION_UNIQUE_ON_NAME_CONSTRAINT_NAME: &str = "unique__permission__name";



#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Permission::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new_with_type(Permission::Id, ColumnType::Integer)
                            .not_null()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new_with_type(Permission::Name, ColumnType::String(None))
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new_with_type(Permission::Description, ColumnType::String(None))
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .name(PERMISSION_PK_CONSTRAINT_NAME)
                            .table(Permission::Table)
                            .col(Permission::Id)
                            .primary(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(PERMISSION_IDX_ON_ID_INDEX_NAME)
                    .table(Permission::Table)
                    .col(Permission::Id)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(PERMISSION_IDX_ON_NAME_INDEX_NAME)
                    .table(Permission::Table)
                    .col(Permission::Name)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(PERMISSION_UNIQUE_ON_NAME_CONSTRAINT_NAME)
                    .table(Permission::Table)
                    .col(Permission::Name)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Permission::Table).to_owned())
            .await
    }
}
