use serde::Deserialize;
use uuid::Uuid;


#[derive(Deserialize, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde_impls_for_client_on_models", derive(serde::Serialize))]
pub struct GiveAdministratorRoleRequest {
    pub user_id: Uuid,
}


#[derive(Deserialize, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde_impls_for_client_on_models", derive(serde::Serialize))]
pub struct ResetUserRolesRequest {
    pub user_id: Uuid,
}
