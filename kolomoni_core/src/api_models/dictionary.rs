use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::id::CategoryId;


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct Category {
    pub id: CategoryId,

    pub slovene_name: String,

    pub english_name: String,

    pub created_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,
}
