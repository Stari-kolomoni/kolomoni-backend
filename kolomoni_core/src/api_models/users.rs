use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::id::UserId;



/// User login information.
#[derive(Deserialize, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "more_serde_impls", derive(Serialize))]
#[schema(
    example = json!({
        "username": "sample_user",
        "password": "verysecurepassword" 
    })
)]
pub struct UserLoginRequest {
    /// Username to log in as.
    pub username: String,

    /// Password.
    pub password: String,
}




/// Information with which to refresh a user's login, generating a new access token.
#[derive(Deserialize, ToSchema)]
#[schema(
    example = json!({
        "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJTdGFyaSBLb2xvbW9uaSIsInN\
                          1YiI6IkFQSSB0b2tlbiIsImlhdCI6MTY4Nzk3MTMyMiwiZXhwIjoxNjg4NTc2MTI2LCJ1c2V\
                          ybmFtZSI6InRlc3QiLCJ0b2tlbl90eXBlIjoicmVmcmVzaCJ9.Ze6DI5EZ-swXRQrMW3NIpp\
                          YejclGbyI9D6zmYBWJMLk"
    })
)]
pub struct UserLoginRefreshRequest {
    /// Refresh token to use to generate an access token.
    ///
    /// Token must not have expired to work.
    pub refresh_token: String,
}




/// Response on successful login refresh.
#[derive(Serialize, Debug, ToSchema)]
#[schema(
    example = json!({
        "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJTdGFyaSBLb2xvbW9uaSIsInN1\
                         YiI6IkFQSSB0b2tlbiIsImlhdCI6MTY4Nzk3MTMyMiwiZXhwIjoxNjg4MDU3NzI2LCJ1c2Vyb\
                         mFtZSI6InRlc3QiLCJ0b2tlbl90eXBlIjoiYWNjZXNzIn0.ZnuhEVacQD_pYzkW9h6aX3eoRN\
                         OAs2-y3EngGBglxkk"
    })
)]
pub struct UserLoginRefreshResponse {
    /// Newly-generated access token to use in future requests.
    pub access_token: String,
}




/// Response on successful user login.
///
/// Contains two tokens:
/// - the `access_token` that should be appended to future requests and
/// - the `refresh_token` that can be used on `POST /api/v1/users/login/refresh` to
///   receive a new (fresh) request token.
///
/// This works because the `refresh_token` has a longer expiration time.
#[derive(Serialize, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "more_serde_impls", derive(serde::Deserialize))]
#[schema(
    example = json!({
        "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJTdGFyaSBLb2xvbW9uaSIsInN1Y\
                         iI6IkFQSSB0b2tlbiIsImlhdCI6MTY4Nzk3MTMyMiwiZXhwIjoxNjg4MDU3NzI2LCJ1c2VybmF\
                         tZSI6InRlc3QiLCJ0b2tlbl90eXBlIjoiYWNjZXNzIn0.ZnuhEVacQD_pYzkW9h6aX3eoRNOAs\
                         2-y3EngGBglxkk",
        "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJTdGFyaSBLb2xvbW9uaSIsInN1\
                          YiI6IkFQSSB0b2tlbiIsImlhdCI6MTY4Nzk3MTMyMiwiZXhwIjoxNjg4NTc2MTI2LCJ1c2Vyb\
                          mFtZSI6InRlc3QiLCJ0b2tlbl90eXBlIjoicmVmcmVzaCJ9.Ze6DI5EZ-swXRQrMW3NIppYej\
                          clGbyI9D6zmYBWJMLk"
    })
)]
pub struct UserLoginResponse {
    /// JWT access token.
    /// Provide in subsequent requests in the `Authorization` header as `Bearer your_token_here`.
    pub access_token: String,

    /// JWT refresh token.
    pub refresh_token: String,
}




/// Information about a single user.
///
/// This struct is used as part of a response in the public API.
///
/// TODO needs updated example
#[derive(Serialize, PartialEq, Eq, Clone, Debug, ToSchema)]
#[cfg_attr(feature = "more_serde_impls", derive(Deserialize))]
#[schema(example = json!({
    "id": 1,
    "username": "janeznovak",
    "display_name": "Janez Novak",
    "joined_at": "2023-06-27T20:33:53.078789Z",
    "last_modified_at": "2023-06-27T20:34:27.217273Z",
    "last_active_at": "2023-06-27T20:34:27.253746Z"
}))]
pub struct UserInfo {
    /// Internal user ID.
    pub id: UserId,

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




/// Information about one user in particular.
///
/// This struct is used as a response in the public API.
#[derive(Serialize, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "more_serde_impls", derive(Deserialize))]
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
    pub user: UserInfo,
}




/// User (API caller) request to change a user's display name.
///
/// This struct is used as a request in the public API.
#[derive(Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
#[cfg_attr(feature = "more_serde_impls", derive(Serialize))]
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
#[cfg_attr(feature = "more_serde_impls", derive(Deserialize))]
pub struct UserDisplayNameChangeResponse {
    pub user: UserInfo,
}


#[derive(Serialize, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "more_serde_impls", derive(Deserialize))]
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




/// Response containing a list of active permissions.
///
/// This struct is used as a response in the public API.
#[derive(Serialize, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "more_serde_impls", derive(Deserialize))]
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
    pub permissions: Vec<&'static str>,
}




/// List of registered users.
#[derive(Serialize, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "more_serde_impls", derive(serde::Deserialize))]
#[schema(example = json!({
    "users": [
        {
            "id": 1,
            "username": "janeznovak",
            "display_name": "Janez Novak",
            "joined_at": "2023-06-27T20:33:53.078789Z",
            "last_modified_at": "2023-06-27T20:34:27.217273Z",
            "last_active_at": "2023-06-27T20:34:27.253746Z"
        },
    ]
}))]
pub struct RegisteredUsersListResponse {
    pub users: Vec<UserInfo>,
}




/// User registration request provided by an API caller.
#[derive(Deserialize, Clone, Debug, ToSchema)]
#[schema(example = json!({
    "username": "janeznovak",
    "display_name": "Janez Novak",
    "password": "perica_reže_raci_rep"
}))]
#[cfg_attr(feature = "more_serde_impls", derive(serde::Serialize))]
pub struct UserRegistrationRequest {
    /// Username to register as (not the same as the display name).
    pub username: String,

    /// Name to display as in the UI.
    pub display_name: String,

    /// Password for this user account.
    pub password: String,
}




/// API-serializable response upon successful user registration.
/// Contains the newly-created user's information.
#[derive(Serialize, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "more_serde_impls", derive(serde::Deserialize))]
#[schema(
    example = json!({
        "user": {
            "id": 1,
            "username": "janeznovak",
            "display_name": "Janez Novak",
            "joined_at": "2023-06-27T20:33:53.078789Z",
            "last_modified_at": "2023-06-27T20:34:27.217273Z",
            "last_active_at": "2023-06-27T20:34:27.253746Z"
        }
    })
)]
pub struct UserRegistrationResponse {
    pub user: UserInfo,
}



#[derive(Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
#[cfg_attr(feature = "more_serde_impls", derive(serde::Serialize))]
#[schema(
    example = json!({
        "roles_to_add": ["administrator"]
    })
)]
pub struct UserRoleAddRequest {
    pub roles_to_add: Vec<String>,
}


#[derive(Deserialize, PartialEq, Eq, Debug, ToSchema)]
#[cfg_attr(feature = "more_serde_impls", derive(serde::Serialize))]
#[schema(
    example = json!({
        "roles_to_remove": ["administrator"]
    })
)]
pub struct UserRoleRemoveRequest {
    pub roles_to_remove: Vec<String>,
}
