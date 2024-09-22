use kolomoni_core::id::{RoleId, UserId};
use uuid::Uuid;


pub struct UserRoleModel {
    pub user_id: UserId,

    pub role_id: RoleId,
}

#[allow(dead_code)]
pub struct InternalUserRoleModel {
    pub(crate) user_id: Uuid,

    pub(crate) role_id: i32,
}
