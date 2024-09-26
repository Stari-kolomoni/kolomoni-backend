use serde::{Deserialize, Serialize};
use uuid::Uuid;


macro_rules! impl_uuid_display_for_newtype_struct {
    ($struct_type:ty) => {
        impl std::fmt::Display for $struct_type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                uuid::fmt::Simple::from_uuid(self.0).fmt(f)
            }
        }
    };
}

macro_rules! impl_transparent_display_for_newtype_struct {
    ($struct_type:ty) => {
        impl std::fmt::Display for $struct_type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}




#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct CategoryId(#[serde(with = "uuid::serde::simple")] pub(crate) Uuid);

impl CategoryId {
    #[inline]
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    #[inline]
    pub fn generate() -> Self {
        Self(Uuid::now_v7())
    }

    #[inline]
    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl_uuid_display_for_newtype_struct!(CategoryId);



#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct EditId(#[serde(with = "uuid::serde::simple")] pub(crate) Uuid);

impl EditId {
    #[inline]
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    #[inline]
    pub fn into_inner(self) -> Uuid {
        self.0
    }
}

impl_uuid_display_for_newtype_struct!(EditId);



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




#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserId(#[serde(with = "uuid::serde::simple")] pub(crate) Uuid);

impl UserId {
    #[inline]
    pub const fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    #[inline]
    pub const fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl_uuid_display_for_newtype_struct!(UserId);




#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct WordId(#[serde(with = "uuid::serde::simple")] pub(crate) Uuid);

impl WordId {
    #[inline]
    pub const fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    #[inline]
    pub const fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl_uuid_display_for_newtype_struct!(WordId);




#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct EnglishWordId(#[serde(with = "uuid::serde::simple")] pub(crate) Uuid);

impl EnglishWordId {
    #[inline]
    pub const fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    #[inline]
    pub fn generate() -> Self {
        Self(Uuid::now_v7())
    }

    #[inline]
    pub fn into_word_id(self) -> WordId {
        WordId::new(self.0)
    }

    #[inline]
    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl_uuid_display_for_newtype_struct!(EnglishWordId);




#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct SloveneWordId(#[serde(with = "uuid::serde::simple")] pub(crate) Uuid);

impl SloveneWordId {
    #[inline]
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    #[inline]
    pub fn generate() -> Self {
        Self(Uuid::now_v7())
    }

    #[inline]
    pub fn into_word_id(self) -> WordId {
        WordId::new(self.0)
    }

    #[inline]
    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl_uuid_display_for_newtype_struct!(SloveneWordId);




#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct WordMeaningId(#[serde(with = "uuid::serde::simple")] pub(crate) Uuid);

impl WordMeaningId {
    #[inline]
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    #[inline]
    pub fn into_inner(self) -> Uuid {
        self.0
    }
}

impl_uuid_display_for_newtype_struct!(WordMeaningId);




#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct EnglishWordMeaningId(#[serde(with = "uuid::serde::simple")] pub(crate) Uuid);

impl EnglishWordMeaningId {
    #[inline]
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    #[inline]
    pub fn generate() -> Self {
        Self(Uuid::now_v7())
    }

    #[inline]
    pub fn into_word_meaning_id(self) -> WordMeaningId {
        WordMeaningId::new(self.0)
    }

    #[inline]
    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl_uuid_display_for_newtype_struct!(EnglishWordMeaningId);




#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct SloveneWordMeaningId(#[serde(with = "uuid::serde::simple")] pub(crate) Uuid);

impl SloveneWordMeaningId {
    #[inline]
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    #[inline]
    pub fn generate() -> Self {
        Self(Uuid::now_v7())
    }

    #[inline]
    pub fn into_word_meaning_id(self) -> WordMeaningId {
        WordMeaningId::new(self.0)
    }

    #[inline]
    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl_uuid_display_for_newtype_struct!(SloveneWordMeaningId);
