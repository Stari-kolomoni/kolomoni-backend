use std::str::FromStr;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;


macro_rules! impl_transparent_display_for_newtype_struct {
    ($struct_type:ty) => {
        impl std::fmt::Display for $struct_type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}


pub trait KolomoniUuidNewtype: FromStr {}


macro_rules! create_uuid_newtype {
    ($struct_name:ident) => {
        #[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
        #[derive(serde::Serialize, serde::Deserialize)]
        #[derive(utoipa::ToSchema)]
        #[serde(transparent)]
        #[schema(value_type = uuid::Uuid)]
        #[schema(format = Uuid)]
        pub struct $struct_name(#[serde(with = "uuid::serde::simple")] pub(crate) uuid::Uuid);

        impl $struct_name {
            #[inline]
            pub fn new(uuid: uuid::Uuid) -> Self {
                Self(uuid)
            }

            #[inline]
            pub fn generate() -> Self {
                Self(uuid::Uuid::now_v7())
            }

            #[inline]
            pub fn into_uuid(self) -> uuid::Uuid {
                self.0
            }
        }

        impl std::str::FromStr for $struct_name {
            type Err = uuid::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let inner_uuid = <uuid::Uuid as std::str::FromStr>::from_str(s)?;

                Ok(Self(inner_uuid))
            }
        }

        impl $crate::ids::KolomoniUuidNewtype for $struct_name {}

        impl std::fmt::Display for $struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                uuid::fmt::Hyphenated::from_uuid(self.0).fmt(f)
            }
        }

        impl std::fmt::Debug for $struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    concat!(stringify!($struct_name), "<{}>"),
                    self.0
                )
            }
        }
    };
}

create_uuid_newtype!(CategoryId);

create_uuid_newtype!(EditId);

create_uuid_newtype!(UserId);


create_uuid_newtype!(WordId);

impl WordId {
    /// "Downcasts" a [`WordId`] into an [`EnglishWordId`],
    /// forcefully assigning the English language to this word.
    ///
    /// **It is up to the caller to ensure this is — semantically — a valid conversion.
    /// Newtypes, such as [`WordId`], are exposed through the public API of the sub-crates
    /// precisely because of increased type safety, so if you find yourself having to
    /// call this function outside of the e.g. [`kolomoni_core`] or [`kolomoni_database`] crates,
    /// think very carefully about whether this conversion is semantically valid.**
    pub fn downcast_to_english_word_id_unchecked(self) -> EnglishWordId {
        EnglishWordId::new(self.0)
    }

    /// "Downcasts" a [`WordId`] into a [`SloveneWordId`],
    /// forcefully assigning the Slovene language to this word.
    ///
    /// **It is up to the caller to ensure this is — semantically — a valid conversion.
    /// Newtypes, such as [`WordId`], are exposed through the public API of the sub-crates
    /// precisely because of increased type safety, so if you find yourself having to
    /// call this function outside of the e.g. [`kolomoni_core`] or [`kolomoni_database`] crates,
    /// think very carefully about whether this conversion is semantically valid.**
    pub fn downcast_to_slovene_word_id_unchecked(self) -> SloveneWordId {
        SloveneWordId::new(self.0)
    }
}


create_uuid_newtype!(WordMeaningId);

impl WordMeaningId {
    /// "Downcasts" a [`WordMeaningId`] into an [`EnglishWordMeaningId`],
    /// forcefully assigning the English language to this word meaning.
    ///
    /// **It is up to the caller to ensure this is — semantically — a valid conversion.
    /// Newtypes, such as [`WordMeaningId`], are exposed through the public API of the sub-crates
    /// precisely because of increased type safety, so if you find yourself having to
    /// call this function outside of the e.g. [`kolomoni_core`] or [`kolomoni_database`] crates,
    /// think very carefully about whether this conversion is semantically valid.**
    pub fn downcast_to_english_word_meaning_id_unchecked(self) -> EnglishWordMeaningId {
        EnglishWordMeaningId::new(self.0)
    }

    /// "Downcasts" a [`WordMeaningId`] into an [`SloveneWordMeaningId`],
    /// forcefully assigning the Slovene language to this word meaning.
    ///
    /// **It is up to the caller to ensure this is — semantically — a valid conversion.
    /// Newtypes, such as [`WordMeaningId`], are exposed through the public API of the sub-crates
    /// precisely because of increased type safety, so if you find yourself having to
    /// call this function outside of the e.g. [`kolomoni_core`] or [`kolomoni_database`] crates,
    /// think very carefully about whether this conversion is semantically valid.**
    pub fn downcast_to_slovene_word_meaning_id_unchecked(self) -> SloveneWordMeaningId {
        SloveneWordMeaningId::new(self.0)
    }
}



create_uuid_newtype!(EnglishWordId);

impl EnglishWordId {
    /// "Upcasts" an [`EnglishWordId`] into a [`WordId`],
    /// removing the language information.
    /// This is a lossy operation by definition!
    #[inline]
    pub fn upcast_to_word_id(self) -> WordId {
        WordId::new(self.0)
    }
}



create_uuid_newtype!(EnglishWordMeaningId);

impl EnglishWordMeaningId {
    /// "Upcasts" an [`EnglishWordMeaningId`] into a [`WordMeaningId`],
    /// removing the language information.
    /// This is a lossy operation by definition!
    #[inline]
    pub fn upcast_to_word_meaning_id(self) -> WordMeaningId {
        WordMeaningId::new(self.0)
    }
}



create_uuid_newtype!(SloveneWordId);

impl SloveneWordId {
    /// "Upcast" an [`SloveneWordId`] into a [`WordId`],
    /// removing the language information.
    /// This is a lossy operation by definition!
    #[inline]
    pub fn upcast_to_word_id(self) -> WordId {
        WordId::new(self.0)
    }
}



create_uuid_newtype!(SloveneWordMeaningId);

impl SloveneWordMeaningId {
    /// "Upcast" an [`SloveneWordMeaningId`] into a [`WordMeaningId`],
    /// removing the language information.
    /// This is a lossy operation by definition!
    #[inline]
    pub fn upcast_to_word_meaning_id(self) -> WordMeaningId {
        WordMeaningId::new(self.0)
    }
}




#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, ToSchema)]
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct PermissionId(pub(crate) i32);

impl PermissionId {
    #[inline]
    pub fn new(id: i32) -> Self {
        Self(id)
    }

    #[inline]
    pub fn into_inner(self) -> i32 {
        self.0
    }
}

impl_transparent_display_for_newtype_struct!(PermissionId);



#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, ToSchema)]
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct RoleId(pub(crate) i32);

impl RoleId {
    #[inline]
    pub fn new(id: i32) -> Self {
        Self(id)
    }

    #[inline]
    pub fn into_inner(self) -> i32 {
        self.0
    }
}

impl_transparent_display_for_newtype_struct!(RoleId);
