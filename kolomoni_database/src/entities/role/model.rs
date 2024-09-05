#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
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


pub struct FullModel {
    pub id: i32,

    pub key: String,

    pub description_en: String,

    pub description_sl: String,
}

pub struct ReducedModel {
    pub id: i32,

    pub key: String,
}
