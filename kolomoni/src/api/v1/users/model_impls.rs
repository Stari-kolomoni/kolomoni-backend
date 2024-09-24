use kolomoni_core::api_models::{
    RegisteredUsersListResponse,
    UserDisplayNameChangeResponse,
    UserInfo,
    UserInfoResponse,
    UserPermissionsResponse,
    UserRegistrationResponse,
    UserRolesResponse,
};
use kolomoni_database::entities;

use crate::{api::traits::IntoApiModel, impl_json_response_builder};



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


impl_json_response_builder!(UserInfoResponse);
impl_json_response_builder!(UserDisplayNameChangeResponse);
impl_json_response_builder!(UserRolesResponse);
impl_json_response_builder!(UserPermissionsResponse);

impl_json_response_builder!(RegisteredUsersListResponse);

impl_json_response_builder!(UserRegistrationResponse);
