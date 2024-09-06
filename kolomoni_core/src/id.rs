use uuid::Uuid;


macro_rules! impl_ser_de_for_newtype_struct {
    ($struct_type:ty, $inner_type:ty) => {
        impl serde::Serialize for $struct_type {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                self.0.serialize(serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $struct_type {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Ok(Self(
                    <$inner_type as serde::Deserialize>::deserialize(deserializer)?,
                ))
            }
        }
    };
}


#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CategoryId(pub(crate) Uuid);

impl CategoryId {
    #[inline]
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    #[inline]
    pub fn into_inner(self) -> Uuid {
        self.0
    }
}

impl_ser_de_for_newtype_struct!(CategoryId, Uuid);



#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EditId(pub(crate) Uuid);

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

impl_ser_de_for_newtype_struct!(EditId, Uuid);



#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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

impl_ser_de_for_newtype_struct!(PermissionId, i32);



#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RoleId(i32);

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

impl_ser_de_for_newtype_struct!(RoleId, i32);



#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UserId(Uuid);

impl UserId {
    #[inline]
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    #[inline]
    pub fn into_inner(self) -> Uuid {
        self.0
    }
}

impl_ser_de_for_newtype_struct!(UserId, Uuid);



#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WordId(pub(crate) Uuid);

impl WordId {
    #[inline]
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    #[inline]
    pub fn into_inner(self) -> Uuid {
        self.0
    }
}

impl_ser_de_for_newtype_struct!(WordId, Uuid);



#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EnglishWordId(Uuid);

impl EnglishWordId {
    #[inline]
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    #[inline]
    pub fn into_word_id(self) -> WordId {
        WordId::new(self.0)
    }

    #[inline]
    pub fn into_inner(self) -> Uuid {
        self.0
    }
}

impl_ser_de_for_newtype_struct!(EnglishWordId, Uuid);



#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SloveneWordId(Uuid);

impl SloveneWordId {
    #[inline]
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    #[inline]
    pub fn into_word_id(self) -> WordId {
        WordId::new(self.0)
    }

    #[inline]
    pub fn into_inner(self) -> Uuid {
        self.0
    }
}

impl_ser_de_for_newtype_struct!(SloveneWordId, Uuid);



#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WordMeaningId(Uuid);

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

impl_ser_de_for_newtype_struct!(WordMeaningId, Uuid);



#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EnglishWordMeaningId(Uuid);

impl EnglishWordMeaningId {
    #[inline]
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
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

impl_ser_de_for_newtype_struct!(EnglishWordMeaningId, Uuid);



#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SloveneWordMeaningId(Uuid);

impl SloveneWordMeaningId {
    #[inline]
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
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

impl_ser_de_for_newtype_struct!(SloveneWordMeaningId, Uuid);
