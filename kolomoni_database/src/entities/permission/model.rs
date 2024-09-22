use kolomoni_core::id::PermissionId;

pub struct PermissionModel {
    /// Internal ID of the permission, don't expose externally.
    pub id: PermissionId,

    pub key: String,

    pub description_en: String,

    pub description_sl: String,
}
