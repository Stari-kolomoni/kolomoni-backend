use serde::{Deserialize, Serialize};

use crate::ids::UserId;


#[derive(Deserialize, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde_impls_for_client_on_models", derive(Serialize))]
pub struct GiveAdministratorRoleRequest {
    pub user_id: UserId,
}


#[derive(Deserialize, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde_impls_for_client_on_models", derive(Serialize))]
pub struct ResetUserRolesRequest {
    pub user_id: UserId,
}
