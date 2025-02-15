use kolomoni_core::api_models::{
    ErrorReason,
    RegisteredUsersListResponse,
    UserDisplayNameChangeRequest,
    UserDisplayNameChangeResponse,
    UserInfo,
    UserInfoResponse,
    UserPermissionsResponse,
    UserRoleAddRequest,
    UserRoleRemoveRequest,
    UserRolesResponse,
    UsersErrorReason,
};
use kolomoni_core::ids::UserId;
use kolomoni_core::permissions::PermissionSet;
use kolomoni_core::roles::RoleSet;
use reqwest::StatusCode;
use thiserror::Error;

use crate::errors::ClientError;
use crate::errors::ClientResult;
use crate::macros::{
    handle_error_reasons_or_catch_unexpected_status,
    handle_uncaught_status_code,
    handle_unexpected_error_reason,
    handlers,
};
use crate::request::ApiClientRequestBuild;
use crate::{AuthenticatedHttpClient, Client, UnauthenticatedHttpClient};

pub mod current;


async fn get_all_registered_users<C>(client: &C) -> ClientResult<Vec<UserInfo>>
where
    C: AuthenticatedHttpClient,
{
    let response = client
        .get_request_builder()
        .endpoint_url("/users")
        .send_authenticated()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        let registered_users_response = response.json::<RegisteredUsersListResponse>().await?;

        Ok(registered_users_response.users)
    } else {
        handle_uncaught_status_code!(response_status);
    }
}


#[derive(Debug, Error)]
pub enum UserDataFetchError {
    #[error("a user with the specified ID does not exist")]
    NotFound,

    #[error(transparent)]
    ClientError {
        #[from]
        error: ClientError,
    },
}


async fn get_user_information_by_user_id<C>(
    client: &C,
    user_id: UserId,
) -> ClientResult<UserInfo, UserDataFetchError>
where
    C: UnauthenticatedHttpClient,
{
    let response = client
        .get_request_builder()
        .endpoint_url(format!("/users/{}", user_id))
        .send_unauthenticated()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        let user_info_response = response.json::<UserInfoResponse>().await?;

        Ok(user_info_response.user)
    } else if response_status == StatusCode::NOT_FOUND {
        let user_error_reason = response.users_error_reason().await?;

        match user_error_reason {
            UsersErrorReason::UserNotFound => Err(UserDataFetchError::NotFound),
            _ => handle_unexpected_error_reason!(user_error_reason, response_status),
        }
    } else {
        handle_uncaught_status_code!(response_status);
    }
}



async fn get_user_roles_by_user_id<C>(
    client: &C,
    user_id: UserId,
) -> ClientResult<RoleSet, UserDataFetchError>
where
    C: UnauthenticatedHttpClient,
{
    let response = client
        .get_request_builder()
        .endpoint_url(format!("/users/{}/roles", user_id))
        .send_unauthenticated()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        let user_roles_response = response.json::<UserRolesResponse>().await?;

        let Ok(user_role_set) = RoleSet::try_from_role_names(user_roles_response.role_names) else {
            return Err(ClientError::unexpected_response(
                response_status,
                "server responded with a role set that contains at least one invalid role",
            )
            .into());
        };

        Ok(user_role_set)
    } else if response_status == StatusCode::NOT_FOUND {
        let user_error_reason = response.users_error_reason().await?;

        match user_error_reason {
            UsersErrorReason::UserNotFound => Err(UserDataFetchError::NotFound),
            _ => handle_unexpected_error_reason!(user_error_reason, response_status),
        }
    } else {
        handle_uncaught_status_code!(response_status);
    }
}


async fn get_user_effective_permissions_by_user_id<C>(
    client: &C,
    user_id: UserId,
) -> ClientResult<PermissionSet, UserDataFetchError>
where
    C: UnauthenticatedHttpClient,
{
    let response = client
        .get_request_builder()
        .endpoint_url(format!("/users/{}/permissions", user_id))
        .send_unauthenticated()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        let user_permissions_response = response.json::<UserPermissionsResponse>().await?;

        let Ok(user_permission_set) =
            PermissionSet::try_from_permission_names(user_permissions_response.permissions)
        else {
            return Err(ClientError::unexpected_response(
                response_status,
                "server responded with a permission set that contains at least one invalid permission",
            ).into());
        };

        Ok(user_permission_set)
    } else if response_status == StatusCode::NOT_FOUND {
        let user_error_reason = response.users_error_reason().await?;

        match user_error_reason {
            UsersErrorReason::UserNotFound => Err(UserDataFetchError::NotFound),
            _ => handle_unexpected_error_reason!(user_error_reason, response_status),
        }
    } else {
        handle_uncaught_status_code!(response_status);
    }
}



#[derive(Debug, Error)]
pub enum UserDisplayNameUpdateError {
    #[error("a user with the specified ID does not exist")]
    NotFound,

    #[error("you cannot modify your own account on this endpoint")]
    CannotModifyYourself,

    #[error("the specified display name cannot be set, because it is already in use")]
    DisplayNameAlreadyExists,

    #[error(transparent)]
    ClientError {
        #[from]
        error: ClientError,
    },
}



async fn update_user_display_name_by_user_id<C>(
    client: &C,
    user_id: UserId,
    new_display_name: &str,
) -> ClientResult<UserInfo, UserDisplayNameUpdateError>
where
    C: AuthenticatedHttpClient,
{
    let response = client
        .patch_request_builder()
        .endpoint_url(format!("/users/{}/display_name", user_id))
        .json(&UserDisplayNameChangeRequest {
            new_display_name: new_display_name.to_string(),
        })
        .send_authenticated()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        let user_display_name_change_response =
            response.json::<UserDisplayNameChangeResponse>().await?;

        Ok(user_display_name_change_response.user)
    } else if response_status == StatusCode::FORBIDDEN {
        let error_reason = response.error_reason().await?;

        if let ErrorReason::Users(users_error_reason) = &error_reason {
            if *users_error_reason == UsersErrorReason::CannotModifyYourOwnAccount {
                return Err(UserDisplayNameUpdateError::CannotModifyYourself);
            }
        }

        handle_error_reasons_or_catch_unexpected_status!(
            { reason: error_reason, status_code: response_status },
            [handlers::MissingPermissions]
        );
    } else if response_status == StatusCode::NOT_FOUND {
        let user_error_reason = response.users_error_reason().await?;

        match user_error_reason {
            UsersErrorReason::UserNotFound => Err(UserDisplayNameUpdateError::NotFound),
            _ => handle_unexpected_error_reason!(user_error_reason, response_status),
        }
    } else if response_status == StatusCode::CONFLICT {
        let user_error_reason = response.users_error_reason().await?;

        match user_error_reason {
            UsersErrorReason::DisplayNameAlreadyExists => {
                Err(UserDisplayNameUpdateError::DisplayNameAlreadyExists)
            }
            _ => handle_unexpected_error_reason!(user_error_reason, response_status),
        }
    } else {
        handle_uncaught_status_code!(response_status);
    }
}



#[derive(Debug, Error)]
pub enum UserRolesAddError {
    #[error("a user with the specified ID does not exist")]
    UserNotFound,

    #[error("you cannot modify your own account on this endpoint")]
    CannotModifyYourself,

    #[error("you cannot give out roles you do not have")]
    CannotGiveOutRolesYouDontHave,

    #[error(transparent)]
    ClientError {
        #[from]
        error: ClientError,
    },
}



async fn add_roles_to_user_by_user_id<C>(
    client: &C,
    user_id: UserId,
    roles_to_add: RoleSet,
) -> ClientResult<RoleSet, UserRolesAddError>
where
    C: AuthenticatedHttpClient,
{
    let response = client
        .post_request_builder()
        .endpoint_url(format!("/users/{}/roles", user_id))
        .json(&UserRoleAddRequest {
            roles_to_add: roles_to_add.role_names(),
        })
        .send_authenticated()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        let user_roles_response = response.json::<UserRolesResponse>().await?;

        let Ok(user_role_set) = RoleSet::try_from_role_names(user_roles_response.role_names) else {
            return Err(ClientError::unexpected_response(
                response_status,
                "server responded with a role set that contains at least one invalid role",
            )
            .into());
        };

        Ok(user_role_set)
    } else if response_status == StatusCode::BAD_REQUEST {
        let error_reason = response.error_reason().await?;

        if let ErrorReason::Users(users_error_reason) = &error_reason {
            if matches!(
                users_error_reason,
                UsersErrorReason::InvalidRoleName { .. }
            ) {
                return Err(ClientError::unexpected_response(
                    response_status,
                    "server rejected one of our role names as invalid; \
                    this may indicate the client's available role set is out of date",
                )
                .into());
            }
        }

        handle_error_reasons_or_catch_unexpected_status!(
            { reason: error_reason, status_code: response_status },
            [handlers::InvalidUuidFormat]
        );
    } else if response_status == StatusCode::FORBIDDEN {
        let error_reason = response.error_reason().await?;

        if let ErrorReason::Users(users_error_reason) = &error_reason {
            match users_error_reason {
                UsersErrorReason::CannotModifyYourOwnAccount => {
                    return Err(UserRolesAddError::CannotModifyYourself)
                }
                UsersErrorReason::UnableToGiveOutUnownedRole { .. } => {
                    return Err(UserRolesAddError::CannotGiveOutRolesYouDontHave)
                }
                _ => {}
            }
        }


        handle_error_reasons_or_catch_unexpected_status!(
            { reason: error_reason, status_code: response_status },
            [handlers::MissingPermissions]
        );
    } else if response_status == StatusCode::NOT_FOUND {
        let user_error_reason = response.users_error_reason().await?;

        match user_error_reason {
            UsersErrorReason::UserNotFound => Err(UserRolesAddError::UserNotFound),
            _ => handle_unexpected_error_reason!(user_error_reason, response_status),
        }
    } else {
        handle_uncaught_status_code!(response_status);
    }
}



#[derive(Debug, Error)]
pub enum UserRolesRemoveError {
    #[error("a user with the specified ID does not exist")]
    UserNotFound,

    #[error("you cannot modify your own account on this endpoint")]
    CannotModifyYourself,

    #[error("you cannot take away user roles you do not have")]
    CannotTakeAwayRolesYouDontHave,

    #[error(transparent)]
    ClientError {
        #[from]
        error: ClientError,
    },
}



async fn remove_roles_from_user_by_user_id<C>(
    client: &C,
    user_id: UserId,
    roles_to_remove: RoleSet,
) -> ClientResult<RoleSet, UserRolesRemoveError>
where
    C: AuthenticatedHttpClient,
{
    let response = client
        .delete_request_builder()
        .endpoint_url(format!("/users/{}/roles", user_id))
        .json(&UserRoleRemoveRequest {
            roles_to_remove: roles_to_remove.role_names(),
        })
        .send_authenticated()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        let user_roles_response = response.json::<UserRolesResponse>().await?;

        let Ok(user_role_set) = RoleSet::try_from_role_names(user_roles_response.role_names) else {
            return Err(ClientError::unexpected_response(
                response_status,
                "server responded with a role set that contains at least one invalid role",
            )
            .into());
        };

        Ok(user_role_set)
    } else if response_status == StatusCode::BAD_REQUEST {
        let error_reason = response.error_reason().await?;

        if let ErrorReason::Users(users_error_reason) = &error_reason {
            if matches!(
                users_error_reason,
                UsersErrorReason::InvalidRoleName { .. }
            ) {
                return Err(ClientError::unexpected_response(
                    response_status,
                    "server rejected one of our role names as invalid; \
                    this may indicate the client's available role set is out of date",
                )
                .into());
            }
        }

        handle_error_reasons_or_catch_unexpected_status!(
            { reason: error_reason, status_code: response_status },
            [handlers::InvalidUuidFormat]
        );
    } else if response_status == StatusCode::FORBIDDEN {
        let error_reason = response.error_reason().await?;

        if let ErrorReason::Users(users_error_reason) = &error_reason {
            match users_error_reason {
                UsersErrorReason::CannotModifyYourOwnAccount => {
                    return Err(UserRolesRemoveError::CannotModifyYourself)
                }
                UsersErrorReason::UnableToTakeAwayUnownedRole { .. } => {
                    return Err(UserRolesRemoveError::CannotTakeAwayRolesYouDontHave)
                }
                _ => {}
            }
        }

        handle_error_reasons_or_catch_unexpected_status!(
            { reason: error_reason, status_code: response_status },
            [handlers::MissingPermissions]
        );
    } else if response_status == StatusCode::NOT_FOUND {
        let users_error_reason = response.users_error_reason().await?;

        match users_error_reason {
            UsersErrorReason::UserNotFound => Err(UserRolesRemoveError::UserNotFound),
            _ => handle_unexpected_error_reason!(users_error_reason, response_status),
        }
    } else {
        handle_uncaught_status_code!(response_status);
    }
}




pub struct SpecificUserUnauthenticatedApi<'c, C>
where
    C: Client + UnauthenticatedHttpClient,
{
    client: &'c C,
}

impl<'c, C> SpecificUserUnauthenticatedApi<'c, C>
where
    C: Client + UnauthenticatedHttpClient,
{
    pub(crate) const fn new(client: &'c C) -> Self {
        Self { client }
    }

    pub async fn get_user_information_by_user_id(
        &self,
        user_id: UserId,
    ) -> Result<UserInfo, UserDataFetchError> {
        get_user_information_by_user_id(self.client, user_id).await
    }

    pub async fn get_user_roles_by_user_id(
        &self,
        user_id: UserId,
    ) -> Result<RoleSet, UserDataFetchError> {
        get_user_roles_by_user_id(self.client, user_id).await
    }

    pub async fn get_user_effective_permissions_by_user_id(
        &self,
        user_id: UserId,
    ) -> Result<PermissionSet, UserDataFetchError> {
        get_user_effective_permissions_by_user_id(self.client, user_id).await
    }
}



pub struct AuthenticatedSpecificUserApi<'c, C>
where
    C: Client + UnauthenticatedHttpClient + AuthenticatedHttpClient,
{
    client: &'c C,
}

impl<'c, C> AuthenticatedSpecificUserApi<'c, C>
where
    C: Client + UnauthenticatedHttpClient + AuthenticatedHttpClient,
{
    pub(crate) const fn new(client: &'c C) -> Self {
        Self { client }
    }


    pub async fn get_user_information_by_user_id(
        &self,
        user_id: UserId,
    ) -> Result<UserInfo, UserDataFetchError> {
        get_user_information_by_user_id(self.client, user_id).await
    }

    pub async fn get_user_roles_by_user_id(
        &self,
        user_id: UserId,
    ) -> Result<RoleSet, UserDataFetchError> {
        get_user_roles_by_user_id(self.client, user_id).await
    }

    pub async fn get_user_effective_permissions_by_user_id(
        &self,
        user_id: UserId,
    ) -> Result<PermissionSet, UserDataFetchError> {
        get_user_effective_permissions_by_user_id(self.client, user_id).await
    }


    pub async fn get_all_registered_users(&self) -> Result<Vec<UserInfo>, ClientError> {
        get_all_registered_users(self.client).await
    }

    pub async fn update_user_display_name_by_user_id<U>(
        &self,
        user_id: UserId,
        new_display_name: U,
    ) -> Result<UserInfo, UserDisplayNameUpdateError>
    where
        U: AsRef<str>,
    {
        update_user_display_name_by_user_id(self.client, user_id, new_display_name.as_ref()).await
    }

    pub async fn add_roles_to_user_by_user_id(
        &self,
        user_id: UserId,
        roles_to_add: RoleSet,
    ) -> Result<RoleSet, UserRolesAddError> {
        add_roles_to_user_by_user_id(self.client, user_id, roles_to_add).await
    }

    pub async fn remove_roles_from_user_by_user_id(
        &self,
        user_id: UserId,
        roles_to_remove: RoleSet,
    ) -> Result<RoleSet, UserRolesRemoveError> {
        remove_roles_from_user_by_user_id(self.client, user_id, roles_to_remove).await
    }
}
