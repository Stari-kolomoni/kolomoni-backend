use std::borrow::Cow;

use chrono::{DateTime, Utc};
use kolomoni_core::{
    edit::Edit,
    id::{EditId, UserId},
};
use uuid::Uuid;

use crate::TryIntoModel;



pub struct Model {
    pub id: EditId,

    /// Contains certain duplicated information which we've decided *not* to
    /// deduplicate (with the intent of serialized edit data being completely standalone):
    /// - schema version (`schema_version` field),
    /// - time of edit (`data.authored_at` field), and
    /// - edit author (`data.authored_by` field).
    pub data: Edit,

    pub data_schema_version: u32,

    pub performed_at: DateTime<Utc>,

    pub performed_by: UserId,
}



pub struct IntermediateModel {
    pub id: Uuid,

    pub data: serde_json::Value,

    pub data_schema_version: i32,

    pub performed_at: DateTime<Utc>,

    pub performed_by: Uuid,
}

impl TryIntoModel for IntermediateModel {
    type Model = Model;
    type Error = Cow<'static, str>;

    fn try_into_model(self) -> Result<Self::Model, Self::Error> {
        let id = EditId::new(self.id);

        let data = serde_json::from_value::<Edit>(self.data).map_err(|error| {
            Cow::from(format!(
                "failed to deserialize edit JSON data due to: {:?}",
                error
            ))
        })?;

        let data_schema_version = if self.data_schema_version > 0 {
            self.data_schema_version as u32
        } else {
            return Err("invalid data_schema_version: below or equal 0, expected u32".into());
        };

        let performed_by = UserId::new(self.performed_by);


        Ok(Self::Model {
            id,
            data,
            data_schema_version,
            performed_at: self.performed_at,
            performed_by,
        })
    }
}
