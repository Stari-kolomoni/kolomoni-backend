use chrono::{DateTime, Utc};
use kolomoni_core::ids::UserId;
use uuid::Uuid;

use crate::{IntoExternalModel, IntoInternalModel};



pub struct UserModel {
    /// UUIDv7
    pub id: UserId,

    pub username: String,

    pub display_name: String,

    pub hashed_password: String,

    pub joined_at: DateTime<Utc>,

    pub last_modified_at: DateTime<Utc>,

    pub last_active_at: DateTime<Utc>,
}

impl IntoInternalModel for UserModel {
    type InternalModel = InternalUserModel;

    fn into_internal_model(self) -> Self::InternalModel {
        Self::InternalModel {
            id: self.id.into_uuid(),
            username: self.username,
            display_name: self.display_name,
            hashed_password: self.hashed_password,
            joined_at: self.joined_at,
            last_modified_at: self.last_modified_at,
            last_active_at: self.last_active_at,
        }
    }
}



pub struct InternalUserModel {
    /// UUIDv7
    pub(crate) id: Uuid,

    pub(crate) username: String,

    pub(crate) display_name: String,

    pub(crate) hashed_password: String,

    pub(crate) joined_at: DateTime<Utc>,

    pub(crate) last_modified_at: DateTime<Utc>,

    pub(crate) last_active_at: DateTime<Utc>,
}

impl IntoExternalModel for InternalUserModel {
    type ExternalModel = UserModel;

    fn into_external_model(self) -> Self::ExternalModel {
        let user_id = UserId::new(self.id);

        Self::ExternalModel {
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
