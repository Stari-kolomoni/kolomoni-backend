use std::borrow::BorrowMut;

use sea_orm_migration::prelude::*;



#[derive(DeriveIden)]
enum Word {
    #[sea_orm(iden = "word")]
    Table,

    #[sea_orm(iden = "id")]
    Id,

    #[sea_orm(iden = "language")]
    Language,
}

const WORD_PK_CONSTRAINT_NAME: &str = "pk__word";
const WORD_INDEX_ON_ID: &str = "index__word__on__id";



#[derive(DeriveIden)]
enum WordSlovene {
    #[sea_orm(iden = "word_slovene")]
    Table,

    #[sea_orm(iden = "word_id")]
    WordId,

    #[sea_orm(iden = "lemma")]
    Lemma,

    #[sea_orm(iden = "disambiguation")]
    Disambiguation,

    #[sea_orm(iden = "description")]
    Description,

    #[sea_orm(iden = "added_at")]
    AddedAt,

    #[sea_orm(iden = "last_edited_at")]
    LastEditedAt,
}

const WORD_SLOVENE_PK_CONSTRAINT_NAME: &str = "pk__word_slovene";
const WORD_SLOVENE_INDEX_ON_WORD_ID: &str = "index__word_slovene__on__word_id";
const WORD_SLOVENE_FK_WORD_ID_CONSTRAINT_NAME: &str = "fk__word_slovene__word_id__word";


#[derive(DeriveIden)]
enum WordEnglish {
    #[sea_orm(iden = "word_english")]
    Table,

    #[sea_orm(iden = "word_id")]
    WordId,

    #[sea_orm(iden = "lemma")]
    Lemma,

    #[sea_orm(iden = "disambiguation")]
    Disambiguation,

    #[sea_orm(iden = "description")]
    Description,

    #[sea_orm(iden = "added_at")]
    AddedAt,

    #[sea_orm(iden = "last_edited_at")]
    LastEditedAt,
}

const WORD_ENGLISH_PK_CONSTRAINT_NAME: &str = "pk__word_english";
const WORD_ENGLISH_INDEX_ON_WORD_ID: &str = "index__word_english__on__word_id";
const WORD_ENGLISH_FOREIGN_KEY_WORD_ID: &str = "fk__word_english__word_id__word";




#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Generic word entity

        manager
            .create_table(
                Table::create()
                    .table(Word::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new_with_type(Word::Id, ColumnType::Uuid)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new_with_type(Word::Language, ColumnType::String(Some(12)))
                            .check(Expr::col(Word::Language).in_tuples(["en", "si"]))
                            .not_null(),
                    )
                    .primary_key(Index::create().name(WORD_PK_CONSTRAINT_NAME).col(Word::Id))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(WORD_INDEX_ON_ID)
                    .table(Word::Table)
                    .col(Word::Id)
                    .to_owned(),
            )
            .await?;



        // Slovene word

        manager
            .create_table(
                Table::create()
                    .table(WordSlovene::Table)
                    .if_not_exists()
                    .col(ColumnDef::new_with_type(WordSlovene::WordId, ColumnType::Uuid).not_null())
                    .col(
                        ColumnDef::new_with_type(WordSlovene::Lemma, ColumnType::String(None))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new_with_type(
                            WordSlovene::Disambiguation,
                            ColumnType::String(None),
                        )
                        .borrow_mut(),
                    )
                    .col(
                        ColumnDef::new_with_type(WordSlovene::Description, ColumnType::String(None))
                            .borrow_mut(),
                    )
                    .col(
                        ColumnDef::new_with_type(
                            WordSlovene::AddedAt,
                            ColumnType::TimestampWithTimeZone,
                        )
                        .not_null(),
                    )
                    .col(
                        ColumnDef::new_with_type(
                            WordSlovene::LastEditedAt,
                            ColumnType::TimestampWithTimeZone,
                        )
                        .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .name(WORD_SLOVENE_PK_CONSTRAINT_NAME)
                            .col(WordSlovene::WordId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(WORD_SLOVENE_FK_WORD_ID_CONSTRAINT_NAME)
                            .from(WordSlovene::Table, WordSlovene::WordId)
                            .to(Word::Table, Word::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(WORD_SLOVENE_INDEX_ON_WORD_ID)
                    .table(WordSlovene::Table)
                    .col(WordSlovene::WordId)
                    .to_owned(),
            )
            .await?;



        // English word

        manager
            .create_table(
                Table::create()
                    .table(WordEnglish::Table)
                    .if_not_exists()
                    .col(ColumnDef::new_with_type(WordEnglish::WordId, ColumnType::Uuid).not_null())
                    .col(
                        ColumnDef::new_with_type(WordEnglish::Lemma, ColumnType::String(None))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new_with_type(
                            WordEnglish::Disambiguation,
                            ColumnType::String(None),
                        )
                        .borrow_mut(),
                    )
                    .col(
                        ColumnDef::new_with_type(WordEnglish::Description, ColumnType::String(None))
                            .borrow_mut(),
                    )
                    .col(
                        ColumnDef::new_with_type(
                            WordEnglish::AddedAt,
                            ColumnType::TimestampWithTimeZone,
                        )
                        .not_null(),
                    )
                    .col(
                        ColumnDef::new_with_type(
                            WordEnglish::LastEditedAt,
                            ColumnType::TimestampWithTimeZone,
                        )
                        .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .name(WORD_ENGLISH_PK_CONSTRAINT_NAME)
                            .col(WordEnglish::WordId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(WORD_ENGLISH_FOREIGN_KEY_WORD_ID)
                            .from(WordEnglish::Table, WordEnglish::WordId)
                            .to(Word::Table, Word::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(WORD_ENGLISH_INDEX_ON_WORD_ID)
                    .table(WordEnglish::Table)
                    .col(WordEnglish::WordId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(WordEnglish::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(WordSlovene::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Word::Table).to_owned())
            .await
    }
}
