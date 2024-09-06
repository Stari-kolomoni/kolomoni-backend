use kolomoni_core::id::PermissionId;

pub struct FullModel {
    /// Internal ID of the permission, don't expose externally.
    pub id: PermissionId,

    pub key: String,

    pub description_en: String,

    pub description_sl: String,
}

pub struct ReducedModel {
    /// Internal ID of the permission, don't expose externally.
    pub id: PermissionId,

    pub key: String,
}

// TODO continue from here
