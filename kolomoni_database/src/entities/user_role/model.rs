use uuid::Uuid;

use crate::entities::{role::RoleId, user::UserId};

pub struct Model {
    pub user_id: UserId,

    pub role_id: RoleId,
}

#[allow(dead_code)]
pub(super) struct IntermediateModel {
    pub(super) user_id: Uuid,

    pub(super) role_id: i32,
}
