use actix_web::{web, Scope};
use chrono::{DateTime, Utc};
use kolomoni_database::entities;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use self::all::get_all_registered_users;
use self::current::{
    get_current_user_effective_permissions,
    get_current_user_info,
    get_current_user_roles,
    update_current_user_display_name,
};
use self::registration::register_user;
use self::specific::{
    // add_permissions_to_specific_user,
    add_roles_to_specific_user,
    get_specific_user_effective_permissions,
    get_specific_user_info,
    get_specific_user_roles,
    remove_roles_from_specific_user,
    update_specific_user_display_name,
};
use crate::impl_json_response_builder;

pub mod all;
pub mod current;
pub mod registration;
pub mod specific;



/// Information about a single user.
///
/// This struct is used as part of a response in the public API.
#[derive(Serialize, PartialEq, Eq, Clone, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Deserialize))]
#[schema(example = json!({
    "id": 1,
    "username": "janeznovak",
    "display_name": "Janez Novak",
    "joined_at": "2023-06-27T20:33:53.078789Z",
    "last_modified_at": "2023-06-27T20:34:27.217273Z",
    "last_active_at": "2023-06-27T20:34:27.253746Z"
}))]
pub struct UserInformation {
    /// Internal user ID.
    pub id: i32,

    /// Unique username for login.
    pub username: String,

    /// Unique display name.
    pub display_name: String,

    /// Registration date and time.
    pub joined_at: DateTime<Utc>,

    /// Last modification date and time.
    pub last_modified_at: DateTime<Utc>,

    /// Last activity date and time.
    pub last_active_at: DateTime<Utc>,
}

impl UserInformation {
    /// Convert a user database model into a [`UserInformation`]
    /// that can be safely exposed through the API.
    #[inline]
    pub fn from_user_model(model: entities::user::Model) -> Self {
        Self {
            id: model.id,
            username: model.username,
            display_name: model.display_name,
            joined_at: model.joined_at.with_timezone(&Utc),
            last_modified_at: model.last_modified_at.with_timezone(&Utc),
            last_active_at: model.last_active_at.with_timezone(&Utc),
        }
    }
}



/// Information about one user in particular.
///
/// This struct is used as a response in the public API.
#[derive(Serialize, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Deserialize))]
#[schema(example = json!({
    "user": {
        "id": 1,
        "username": "janeznovak",
        "display_name": "Janez Novak",
        "joined_at": "2023-06-27T20:33:53.078789Z",
        "last_modified_at": "2023-06-27T20:34:27.217273Z",
        "last_active_at": "2023-06-27T20:34:27.253746Z"
    }
}))]
pub struct UserInfoResponse {
    pub user: UserInformation,
}

impl UserInfoResponse {
    pub fn new(model: entities::user::Model) -> Self {
        Self {
            user: UserInformation::from_user_model(model),
        }
    }
}

impl_json_response_builder!(UserInfoResponse);



/// User (API caller) request to change a user's display name.
///
/// This struct is used as a request in the public API.
#[derive(Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Serialize))]
#[schema(
    example = json!({
        "new_display_name": "Janez Novak Veliki"
    })
)]
pub struct UserDisplayNameChangeRequest {
    /// Display name to change to.
    pub new_display_name: String,
}



/// Response indicating successful change of a display name.
/// Contains the updated user information.
///
/// This struct is used as a response in the public API.
#[derive(Serialize, PartialEq, Eq, Clone, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Deserialize))]
pub struct UserDisplayNameChangeResponse {
    pub user: UserInformation,
}

impl_json_response_builder!(UserDisplayNameChangeResponse);




#[derive(Serialize, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Deserialize))]
#[schema(
    example = json!({
        "role_names": [
            "user",
            "administrator"
        ]
    })
)]
pub struct UserRolesResponse {
    pub role_names: Vec<String>,
}

impl_json_response_builder!(UserRolesResponse);



/// Response containing a list of active permissions.
///
/// This struct is used as a response in the public API.
#[derive(Serialize, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Deserialize))]
#[schema(
    example = json!({
        "permissions": [
            "user.self:read",
            "user.self:write",
            "user.any:read"
        ]
    })
)]
pub struct UserPermissionsResponse {
    pub permissions: Vec<String>,
}

impl UserPermissionsResponse {
    pub fn from_permission_names(permission_names: Vec<String>) -> Self {
        Self {
            permissions: permission_names,
        }
    }
}

impl_json_response_builder!(UserPermissionsResponse);


/// Router for all user-related operations.
/// Lives under `/api/v1/users`.
#[rustfmt::skip]
pub fn users_router() -> Scope {
    web::scope("users")
        .service(get_all_registered_users)
        .service(register_user)
        .service(get_current_user_info)
        .service(get_current_user_roles)
        .service(get_current_user_effective_permissions)
        .service(update_current_user_display_name)
        .service(get_specific_user_info)
        .service(get_specific_user_effective_permissions)
        .service(get_specific_user_roles)
        .service(add_roles_to_specific_user)
        .service(remove_roles_from_specific_user)
        .service(update_specific_user_display_name)
}
