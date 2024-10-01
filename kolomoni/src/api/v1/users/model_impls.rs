use kolomoni_core::api_models::UserInfo;
use kolomoni_database::entities;

use crate::api::traits::IntoApiModel;



impl IntoApiModel<UserInfo> for entities::UserModel {
    fn into_api_model(self) -> UserInfo {
        UserInfo {
            id: self.id,
            username: self.username,
            display_name: self.display_name,
            joined_at: self.joined_at,
            last_modified_at: self.last_modified_at,
            last_active_at: self.last_active_at,
        }
    }
}
