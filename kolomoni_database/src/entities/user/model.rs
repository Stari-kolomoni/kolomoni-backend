use chrono::{DateTime, Utc};
use kolomoni_core::id::UserId;
use uuid::Uuid;

use crate::IntoModel;



pub struct Model {
    /// UUIDv7
    pub id: UserId,

    pub username: String,

    pub display_name: String,

    pub hashed_password: String,

    pub joined_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,

    pub last_active_at: DateTime<Utc>,
}


pub(super) struct IntermediateModel {
    /// UUIDv7
    pub id: Uuid,

    pub username: String,

    pub display_name: String,

    pub hashed_password: String,

    pub joined_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,

    pub last_active_at: DateTime<Utc>,
}

impl IntoModel for IntermediateModel {
    type Model = Model;

    fn into_model(self) -> Self::Model {
        let user_id = UserId::new(self.id);

        Self::Model {
            id: user_id,
            username: self.username,
            display_name: self.display_name,
            hashed_password: self.hashed_password,
            joined_at: self.joined_at,
            last_modified_at: self.last_modified_at,
            last_active_at: self.last_active_at,
        }
    }
}
