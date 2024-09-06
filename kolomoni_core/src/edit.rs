use std::u8;

use chrono::{DateTime, Utc};
use serde::{ser::SerializeMap, Deserialize, Serialize};
use serde_with::serde_as;

use crate::id::{
    CategoryId,
    EnglishWordId,
    EnglishWordMeaningId,
    SloveneWordId,
    SloveneWordMeaningId,
    UserId,
};



/// Represents data about a single user-performed edit on Stari Kolomoni.
/// This acts as sort of a record of changes.
///
/// # Internals
/// Serialized / deserialized as a map containing two fields:
/// - a `schema_version` (`u32`), and
/// - a `data` field of the corresponding enum variant's inner type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit {
    VersionOne(VersionOneEdit),
}

impl Edit {
    pub fn schema_version(&self) -> u32 {
        match self {
            Edit::VersionOne(_) => 1,
        }
    }
}


impl Serialize for Edit {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;

        match self {
            Edit::VersionOne(data) => {
                map.serialize_entry("schema_version", &1u32)?;
                map.serialize_entry("data", &data)?;
            }
        };

        map.end()
    }
}


impl<'de> Deserialize<'de> for Edit {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        enum SchemaVersionField {
            VersionOne,
        }


        struct SchemaVersionFieldVisitor;

        impl<'de> serde::de::Visitor<'de> for SchemaVersionFieldVisitor {
            type Value = SchemaVersionField;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a schema_version field")
            }

            fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_u32(v as u32)
            }

            fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_u32(v as u32)
            }

            fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    1u32 => Ok(SchemaVersionField::VersionOne),
                    unrecognized_version => Err(E::invalid_value(
                        serde::de::Unexpected::Unsigned(unrecognized_version as u64),
                        &"known version (1)",
                    )),
                }
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let v_as_u32 = u32::try_from(v).map_err(|_| {
                    E::invalid_value(
                        serde::de::Unexpected::Unsigned(v),
                        &"known version (1)",
                    )
                })?;

                self.visit_u32(v_as_u32)
            }
        }


        impl<'de> serde::Deserialize<'de> for SchemaVersionField {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_u32(SchemaVersionFieldVisitor)
            }
        }


        enum SchemaVersionOrDataField {
            SchemaVersion,
            Data,
        }

        impl<'de> Deserialize<'de> for SchemaVersionOrDataField {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_str(SchemaVersionOrDataFieldVisitor)
            }
        }


        struct SchemaVersionOrDataFieldVisitor;

        impl<'de> serde::de::Visitor<'de> for SchemaVersionOrDataFieldVisitor {
            type Value = SchemaVersionOrDataField;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(
                    "a \"schema_version\" or \"data\" key with u32 or map value, respectively",
                )
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    "schema_version" => Ok(SchemaVersionOrDataField::SchemaVersion),
                    "data" => Ok(SchemaVersionOrDataField::Data),
                    _ => Err(E::invalid_value(
                        serde::de::Unexpected::Str(v),
                        &"\"schema_version\" or \"data\" key",
                    )),
                }
            }
        }



        pub struct EditVisitor;

        impl<'de> serde::de::Visitor<'de> for EditVisitor {
            type Value = Edit;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(
                    "map containing two entries with keys: \"schema_version\" and \"data\"",
                )
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut schema_version: Option<SchemaVersionField> = None;
                let mut data: Option<serde::__private::de::Content> = None;

                while let Some(next_key) = map.next_key::<SchemaVersionOrDataField>()? {
                    match next_key {
                        SchemaVersionOrDataField::SchemaVersion => {
                            if schema_version.is_some() {
                                return Err(serde::de::Error::duplicate_field(
                                    "schema_version",
                                ));
                            }

                            schema_version = Some(map.next_value()?);
                        }
                        SchemaVersionOrDataField::Data => {
                            if data.is_some() {
                                return Err(serde::de::Error::duplicate_field("data"));
                            }

                            data = Some(map.next_value()?);
                        }
                    }
                }


                let Some(schema_version) = schema_version else {
                    return Err(serde::de::Error::missing_field("schema_version"));
                };

                let Some(data) = data else {
                    return Err(serde::de::Error::missing_field("data"));
                };

                let content_deserializer =
                    serde::__private::de::ContentDeserializer::<A::Error>::new(data);

                match schema_version {
                    SchemaVersionField::VersionOne => Ok(Edit::VersionOne(
                        VersionOneEdit::deserialize(content_deserializer)?,
                    )),
                }
            }
        }


        deserializer.deserialize_map(EditVisitor)
    }
}



#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionOneEdit {
    pub authored_by: UserId,

    #[serde_as(as = "serde_with::TimestampMilliSeconds<i64>")]
    pub authored_at: DateTime<Utc>,

    pub action: EditAction,
}




#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "change_type")]
pub enum SloveneWordChange {
    #[serde(rename = "lemma")]
    Lemma {
        previous_lemma: String,
        new_lemma: String,
    },
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "change_type")]
pub enum EnglishWordChange {
    #[serde(rename = "lemma")]
    Lemma {
        previous_lemma: String,
        new_lemma: String,
    },
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "change_type")]
pub enum SloveneWordMeaningChange {
    #[serde(rename = "disambiguation-changed")]
    DisambiguationChanged {
        previous_disambiguation: Option<String>,
        new_disambiguation: Option<String>,
    },

    #[serde(rename = "abbreviation-changed")]
    AbbreviationChanged {
        previous_abbreviation: Option<String>,
        new_abbreviation: Option<String>,
    },

    #[serde(rename = "description-changed")]
    DescriptionChanged {
        previous_description: Option<String>,
        new_description: Option<String>,
    },

    #[serde(rename = "category-added")]
    CategoryAdded { category_id: CategoryId },

    #[serde(rename = "category-removed")]
    CategoryRemoved { category_id: CategoryId },
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "change_type")]
pub enum EnglishWordMeaningChange {
    #[serde(rename = "disambiguation-changed")]
    DisambiguationChanged {
        previous_disambiguation: Option<String>,
        new_disambiguation: Option<String>,
    },

    #[serde(rename = "abbreviation-changed")]
    AbbreviationChanged {
        previous_abbreviation: Option<String>,
        new_abbreviation: Option<String>,
    },

    #[serde(rename = "description-changed")]
    DescriptionChanged {
        previous_description: Option<String>,
        new_description: Option<String>,
    },

    #[serde(rename = "category-added")]
    CategoryAdded { category_id: CategoryId },

    #[serde(rename = "category-removed")]
    CategoryRemoved { category_id: CategoryId },
}



#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EditAction {
    /*
     * Slovene words
     */
    #[serde(rename = "created-slovene-word")]
    CreatedSloveneWord {
        /// ID of the newly-created slovene word.
        id: SloveneWordId,
    },

    #[serde(rename = "updated-slovene-word")]
    UpdatedSloveneWord {
        /// ID of the slovene word that was updated.
        id: SloveneWordId,

        /// One or more slovene word changes that happened as part of this edit.
        changes: Vec<SloveneWordChange>,
    },

    #[serde(rename = "deleted-slovene-word")]
    DeletedSloveneWord {
        /// ID of the slovene word that was just deleted.
        id: SloveneWordId,

        /// Lemma of the slovene word that was just deleted.
        lemma: String,
    },

    /*
     * English words
     */
    #[serde(rename = "created-english-word")]
    CreatedEnglishWord {
        /// ID of the newly-created english word.
        id: EnglishWordId,
    },

    #[serde(rename = "updated-english-word")]
    UpdatedEnglishWord {
        /// ID of the updated english word.
        id: EnglishWordId,

        /// One or more english word changes that happened as part of this edit.
        changes: Vec<EnglishWordChange>,
    },

    #[serde(rename = "deleted-english-word")]
    DeletedEnglishWord {
        /// ID of the english word that was just deleted.
        id: EnglishWordId,

        /// Lemma of the english word that was just deleted.
        lemma: String,
    },
    /*
     * Slovene word meanings (including their category changes, etc.)
     */
    #[serde(rename = "created-slovene-word-meaning")]
    CreatedSloveneWordMeaning {
        /// ID of the newly-created slovene word meaning.
        id: SloveneWordMeaningId,
    },

    #[serde(rename = "updated-slovene-word-meaning")]
    UpdatedSloveneWordMeaning {
        /// ID of the updated slovene word meaning.
        id: SloveneWordMeaningId,

        /// One or more slovene word meaning changes that happened as part of this edit.
        changes: Vec<SloveneWordMeaningChange>,
    },

    #[serde(rename = "deleted-slovene-word-meaning")]
    DeletedSloveneWordMeaning {
        /// ID of the slovene word meaning that was just deleted.
        id: SloveneWordMeaningId,
    },
    /*
     * English word meanings (including their category changes, etc.)
     */
    #[serde(rename = "created-english-word-meaning")]
    CreatedEnglishWordMeaning {
        /// ID of the newly-created english word meaning.
        id: EnglishWordMeaningId,
    },

    #[serde(rename = "updated-english-word-meaning")]
    UpdatedEnglishWordMeaning {
        /// ID of the updated english word meaning.
        id: EnglishWordMeaningId,

        /// One or more english word meaning changes that happened as part of this edit.
        changes: Vec<EnglishWordMeaningChange>,
    },

    #[serde(rename = "deleted-english-word-meaning")]
    DeletedEnglishWordMeaning {
        /// ID of the english word meaning that was just deleted.
        id: EnglishWordMeaningId,
    },
    /*
     * Translations
     */
    #[serde(rename = "created-translation")]
    CreatedTranslation {
        /// ID of the slovene word meaning that is part of the translation relationship.
        slovene_word_meaning_id: SloveneWordMeaningId,

        /// ID of the english word meaning that is part of the translation relationship.
        english_word_meaning_id: EnglishWordMeaningId,
    },

    #[serde(rename = "deleted-translation")]
    DeletedTranslation {
        /// ID of the slovene word meaning that is no longer part of the translation relationship.
        slovene_word_meaning_id: SloveneWordMeaningId,

        /// ID of the english word meaning that is no longer part of the translation relationship.
        english_word_meaning_id: EnglishWordMeaningId,
    },
}



#[cfg(test)]
mod test {
    use uuid::Uuid;

    use super::*;

    #[test]
    fn serializes_as_schema_version_and_data_field() {
        let edit = Edit::VersionOne(VersionOneEdit {
            authored_by: UserId::new(Uuid::now_v7()),
            authored_at: Utc::now(),
            action: EditAction::CreatedEnglishWord {
                id: EnglishWordId::new(Uuid::now_v7()),
            },
        });

        let serialized_edit = serde_json::to_value(&edit).unwrap();
        let serialized_object = serialized_edit.as_object().unwrap();

        assert_eq!(
            serialized_object
                .get("schema_version")
                .unwrap()
                .as_u64()
                .unwrap(),
            1
        );
        assert!(serialized_object.get("data").unwrap().is_object());


        let deserialized_edit: Edit = serde_json::from_value(serialized_edit).unwrap();

        #[allow(irrefutable_let_patterns)]
        let Edit::VersionOne(version_one_deserialized_edit) = deserialized_edit
        else {
            panic!("epxected to deserialize into version one edit");
        };

        #[allow(irrefutable_let_patterns)]
        let Edit::VersionOne(version_one_original_edit) = edit
        else {
            unreachable!();
        };

        assert_eq!(
            version_one_deserialized_edit.action,
            version_one_original_edit.action
        );
        assert_eq!(
            version_one_deserialized_edit.authored_by,
            version_one_original_edit.authored_by
        );
    }
}
