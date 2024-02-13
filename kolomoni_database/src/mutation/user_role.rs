use kolomoni_auth::{Role, RoleSet};
use miette::Result;
use sea_orm::ConnectionTrait;

pub struct UserRoleMutation;

impl UserRoleMutation {
    pub async fn add_roles_to_user<C: ConnectionTrait>(
        database: &C,
        user_id: i32,
        roles: &[Role],
    ) -> Result<RoleSet> {
        todo!();
    }

    pub async fn remove_roles_from_user<C: ConnectionTrait>(
        database: &C,
        user_id: i32,
        roles: &[Role],
    ) -> Result<RoleSet> {
        todo!();
    }
}
