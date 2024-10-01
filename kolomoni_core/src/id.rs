use std::str::FromStr;

use serde::{Deserialize, Serialize};


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
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        #[derive(serde::Serialize, serde::Deserialize)]
        #[serde(transparent)]
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

        impl $crate::id::KolomoniUuidNewtype for $struct_name {}

        impl std::fmt::Display for $struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                uuid::fmt::Simple::from_uuid(self.0).fmt(f)
            }
        }
    };
}



create_uuid_newtype!(CategoryId);

create_uuid_newtype!(EditId);

create_uuid_newtype!(UserId);

create_uuid_newtype!(WordId);

create_uuid_newtype!(WordMeaningId);



create_uuid_newtype!(EnglishWordId);

impl EnglishWordId {
    #[inline]
    pub fn into_word_id(self) -> WordId {
        WordId::new(self.0)
    }
}



create_uuid_newtype!(EnglishWordMeaningId);

impl EnglishWordMeaningId {
    #[inline]
    pub fn into_word_meaning_id(self) -> WordMeaningId {
        WordMeaningId::new(self.0)
    }
}



create_uuid_newtype!(SloveneWordId);

impl SloveneWordId {
    #[inline]
    pub fn into_word_id(self) -> WordId {
        WordId::new(self.0)
    }
}



create_uuid_newtype!(SloveneWordMeaningId);

impl SloveneWordMeaningId {
    #[inline]
    pub fn into_word_meaning_id(self) -> WordMeaningId {
        WordMeaningId::new(self.0)
    }
}




#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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



#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
