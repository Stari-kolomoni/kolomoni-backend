use sea_orm_migration::prelude::*;

use crate::m20240206_234618_create_word_tables::Word;

#[derive(DeriveIden)]
enum Category {
    #[sea_orm(iden = "category")]
    Table,

    #[sea_orm(iden = "id")]
    Id,

    #[sea_orm(iden = "slovene_name")]
    SloveneName,

    #[sea_orm(iden = "english_name")]
    EnglishName,
}

const CATEGORY_PK_CONSTRAINT_NAME: &str = "pk__category";
const CATEGORY_UNIQUE_ON_NAMES_CONSTRAINT_NAME: &str = "unique__category__names";


#[derive(DeriveIden)]
enum WordCategory {
    #[sea_orm(iden = "word_category")]
    Table,

    #[sea_orm(iden = "word_id")]
    WordId,

    #[sea_orm(iden = "category_id")]
    CategoryId,
}

const WORD_CATEGORY_PK_CONSTRAINT_NAME: &str = "pk__word_category";
const WORD_FK_WORD_ID_CONSTRAINT_NAME: &str = "fk__word_category__word_id__word";
const WORD_FK_CATEGORY_ID_CONSTRAINT_NAME: &str = "fk__word_category__category_id__category";



#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Category::Table)
                    .col(ColumnDef::new_with_type(Category::Id, ColumnType::Integer).not_null())
                    .col(
                        ColumnDef::new_with_type(Category::EnglishName, ColumnType::String(None))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new_with_type(Category::SloveneName, ColumnType::String(None))
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .name(CATEGORY_PK_CONSTRAINT_NAME)
                            .col(Category::Id),
                    )
                    .index(
                        Index::create()
                            .name(CATEGORY_UNIQUE_ON_NAMES_CONSTRAINT_NAME)
                            .col(Category::EnglishName)
                            .col(Category::SloveneName)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;


        manager
            .create_table(
                Table::create()
                    .table(WordCategory::Table)
                    .col(ColumnDef::new_with_type(WordCategory::WordId, ColumnType::Uuid).not_null())
                    .col(
                        ColumnDef::new_with_type(WordCategory::CategoryId, ColumnType::Integer)
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .name(WORD_CATEGORY_PK_CONSTRAINT_NAME)
                            .col(WordCategory::WordId)
                            .col(WordCategory::CategoryId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(WORD_FK_WORD_ID_CONSTRAINT_NAME)
                            .from(WordCategory::Table, WordCategory::WordId)
                            .to(Word::Table, Word::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(WORD_FK_CATEGORY_ID_CONSTRAINT_NAME)
                            .from(WordCategory::Table, WordCategory::CategoryId)
                            .to(Category::Table, Category::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(WordCategory::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Category::Table).to_owned())
            .await
    }
}
