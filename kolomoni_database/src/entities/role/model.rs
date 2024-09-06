use kolomoni_core::id::RoleId;

pub struct FullModel {
    pub id: RoleId,

    pub key: String,

    pub description_en: String,

    pub description_sl: String,
}

pub struct ReducedModel {
    pub id: RoleId,

    pub key: String,
}
