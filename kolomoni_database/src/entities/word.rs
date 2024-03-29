//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.12

use sea_orm::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "word"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Eq)]
pub struct Model {
    pub id: Uuid,
    pub language: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    Language,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    Id,
}

impl PrimaryKeyTrait for PrimaryKey {
    type ValueType = Uuid;
    fn auto_increment() -> bool {
        false
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    WordCategory,
    WordEnglish,
    WordSlovene,
}

impl ColumnTrait for Column {
    type EntityName = Entity;
    fn def(&self) -> ColumnDef {
        match self {
            Self::Id => ColumnType::Uuid.def(),
            Self::Language => ColumnType::String(Some(12u32)).def(),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::WordCategory => Entity::has_many(super::word_category::Entity).into(),
            Self::WordEnglish => Entity::has_many(super::word_english::Entity).into(),
            Self::WordSlovene => Entity::has_many(super::word_slovene::Entity).into(),
        }
    }
}

impl Related<super::word_category::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::WordCategory.def()
    }
}

impl Related<super::word_english::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::WordEnglish.def()
    }
}

impl Related<super::word_slovene::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::WordSlovene.def()
    }
}

impl Related<super::category::Entity> for Entity {
    fn to() -> RelationDef {
        super::word_category::Relation::Category.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::word_category::Relation::Word.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
