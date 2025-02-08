pub(crate) mod internal {
    use uuid::Uuid;

    #[allow(dead_code)]
    pub(crate) struct InternalUserRoleModel {
        pub(crate) user_id: Uuid,

        pub(crate) role_id: i32,
    }
}


mod external {
    use kolomoni_core::ids::{RoleId, UserId};

    pub struct UserRoleModel {
        pub user_id: UserId,

        pub role_id: RoleId,
    }
}

pub use external::*;
