use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, Debug, PartialEq, Eq, ToSchema)]
#[cfg_attr(feature = "serde_impls_for_client_on_models", derive(serde::Deserialize))]
pub struct PingResponse {
    pub ok: bool,
}
