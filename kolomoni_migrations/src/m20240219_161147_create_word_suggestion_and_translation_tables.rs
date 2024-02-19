use sea_orm_migration::prelude::*;

use crate::m20240206_234618_create_word_tables::{WordEnglish, WordSlovene};


#[derive(DeriveIden)]
enum WordTranslationSuggestion {
    #[sea_orm(iden = "word_translation_suggestion")]
    Table,

    #[sea_orm(iden = "slovene_word_id")]
    SloveneWordId,

    #[sea_orm(iden = "english_word_id")]
    EnglishWordId,

    #[sea_orm(iden = "suggested_at")]
    SuggestedAt,
}

const SUGGESTION_PK_CONSTRAINT_NAME: &str = "pk__word_translation_suggestion";
const SUGGESTION_FK_SLOVENE_WORD_ID_CONSTRAINT_NAME: &str =
    "fk__word_translation_suggestion__slovene_word_id__word_slovene";
const SUGGESTION_FK_ENGLISH_WORD_ID_CONSTRAINT_NAME: &str =
    "fk__word_translation_suggestion__english_word_id__word_english";
const SUGGESTION_INDEX_ON_BOTH_IDS: &str = "index__word_translation_suggestion__on__both_ids";



#[derive(DeriveIden)]
enum WordTranslation {
    #[sea_orm(iden = "word_translation")]
    Table,

    #[sea_orm(iden = "slovene_word_id")]
    SloveneWordId,

    #[sea_orm(iden = "english_word_id")]
    EnglishWordId,

    #[sea_orm(iden = "translated_at")]
    TranslatedAt,
}

const TRANSLATION_PK_CONSTRAINT_NAME: &str = "pk__word_translation";
const TRANSLATION_FK_SLOVENE_WORD_ID_CONSTRAINT_NAME: &str =
    "fk__word_translation__slovene_word_id__word_slovene";
const TRANSLATION_FK_ENGLISH_WORD_ID_CONSTRAINT_NAME: &str =
    "fk__word_translation__english_word_id__word_english";
const TRANSLATION_INDEX_ON_BOTH_IDS: &str = "index__word_translation__on__both_ids";



#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(WordTranslationSuggestion::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new_with_type(
                            WordTranslationSuggestion::EnglishWordId,
                            ColumnType::Uuid,
                        )
                        .not_null(),
                    )
                    .col(
                        ColumnDef::new_with_type(
                            WordTranslationSuggestion::SloveneWordId,
                            ColumnType::Uuid,
                        )
                        .not_null(),
                    )
                    .col(
                        ColumnDef::new_with_type(
                            WordTranslationSuggestion::SuggestedAt,
                            ColumnType::TimestampWithTimeZone,
                        )
                        .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .name(SUGGESTION_PK_CONSTRAINT_NAME)
                            .col(WordTranslationSuggestion::EnglishWordId)
                            .col(WordTranslationSuggestion::SloveneWordId),
                    )
                    .index(
                        Index::create()
                            .name(SUGGESTION_INDEX_ON_BOTH_IDS)
                            .col(WordTranslationSuggestion::EnglishWordId)
                            .col(WordTranslationSuggestion::SloveneWordId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(SUGGESTION_FK_ENGLISH_WORD_ID_CONSTRAINT_NAME)
                            .from(
                                WordTranslationSuggestion::Table,
                                WordTranslationSuggestion::EnglishWordId,
                            )
                            .to(WordEnglish::Table, WordEnglish::WordId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(SUGGESTION_FK_SLOVENE_WORD_ID_CONSTRAINT_NAME)
                            .from(
                                WordTranslationSuggestion::Table,
                                WordTranslationSuggestion::SloveneWordId,
                            )
                            .to(WordSlovene::Table, WordSlovene::WordId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;


        manager
            .create_table(
                Table::create()
                    .table(WordTranslation::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new_with_type(WordTranslation::EnglishWordId, ColumnType::Uuid)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new_with_type(WordTranslation::SloveneWordId, ColumnType::Uuid)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new_with_type(
                            WordTranslation::TranslatedAt,
                            ColumnType::TimestampWithTimeZone,
                        )
                        .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .name(TRANSLATION_PK_CONSTRAINT_NAME)
                            .col(WordTranslation::EnglishWordId)
                            .col(WordTranslation::SloveneWordId),
                    )
                    .index(
                        Index::create()
                            .name(TRANSLATION_INDEX_ON_BOTH_IDS)
                            .col(WordTranslation::EnglishWordId)
                            .col(WordTranslation::SloveneWordId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(TRANSLATION_FK_ENGLISH_WORD_ID_CONSTRAINT_NAME)
                            .from(
                                WordTranslation::Table,
                                WordTranslation::EnglishWordId,
                            )
                            .to(WordEnglish::Table, WordEnglish::WordId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(TRANSLATION_FK_SLOVENE_WORD_ID_CONSTRAINT_NAME)
                            .from(
                                WordTranslation::Table,
                                WordTranslation::SloveneWordId,
                            )
                            .to(WordSlovene::Table, WordSlovene::WordId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(WordTranslationSuggestion::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(WordTranslation::Table).to_owned())
            .await
    }
}
