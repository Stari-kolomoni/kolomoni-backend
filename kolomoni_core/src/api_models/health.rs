use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, Debug, PartialEq, Eq, ToSchema)]
#[cfg_attr(feature = "more_serde_impls", derive(serde::Deserialize))]
pub struct PingResponse {
    pub ok: bool,
}
