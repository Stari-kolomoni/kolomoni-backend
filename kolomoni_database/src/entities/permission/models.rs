mod external {
    use kolomoni_core::ids::PermissionId;

    pub struct PermissionModel {
        /// Internal ID of the permission! Don't expose externally (e.g. through the REST API).
        pub id: PermissionId,

        pub key: String,

        pub description_en: String,

        pub description_sl: String,
    }
}

pub use external::*;
