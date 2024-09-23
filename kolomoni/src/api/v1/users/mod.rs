use actix_web::{web, Scope};
use kolomoni_core::api_models::{
    UserDisplayNameChangeResponse,
    UserInfo,
    UserInfoResponse,
    UserPermissionsResponse,
    UserRolesResponse,
};
use kolomoni_database::entities;
use registration::register_user;

use self::all::get_all_registered_users;
use self::current::{
    get_current_user_effective_permissions,
    get_current_user_info,
    get_current_user_roles,
    update_current_user_display_name,
};
use self::specific::{
    add_roles_to_specific_user,
    get_specific_user_effective_permissions,
    get_specific_user_info,
    get_specific_user_roles,
    remove_roles_from_specific_user,
    update_specific_user_display_name,
};
use crate::api::traits::IntoApiModel;
use crate::impl_json_response_builder;

pub mod all;
pub mod current;
pub mod registration;
pub mod specific;


impl IntoApiModel for entities::UserModel {
    type ApiModel = UserInfo;

    fn into_api_model(self) -> Self::ApiModel {
        Self::ApiModel {
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




/// Router for all user-related operations.
/// Lives under `/api/v1/users`.
#[rustfmt::skip]
pub fn users_router() -> Scope {
    web::scope("users")
        // all.rs
        .service(get_all_registered_users)
        // registration.rs
        .service(register_user)
        // current.ts
        .service(get_current_user_info)
        .service(get_current_user_roles)
        .service(get_current_user_effective_permissions)
        .service(update_current_user_display_name)
        // specific.rs
        .service(get_specific_user_info)
        .service(get_specific_user_effective_permissions)
        .service(get_specific_user_roles)
        .service(add_roles_to_specific_user)
        .service(remove_roles_from_specific_user)
        .service(update_specific_user_display_name)
}
