use kolomoni_core::api_models::{
    UserDisplayNameChangeRequest,
    UserDisplayNameChangeResponse,
    UserInfo,
    UserInfoResponse,
    UserPermissionsResponse,
    UserRolesResponse,
    UsersErrorReason,
};
use kolomoni_core::permissions::PermissionSet;
use kolomoni_core::roles::RoleSet;
use reqwest::StatusCode;
use thiserror::Error;

use crate::errors::{ClientError, ClientResult};
use crate::macros::{handle_uncaught_status_code, handle_unexpected_error_reason};
use crate::request::ApiClientRequestBuild;
use crate::{AuthenticatedHttpClient, Client};



#[derive(Debug, Error)]
pub enum CurrentUserReadError {
    #[error(
        "the current user does not exist \
        (perhaps you have a valid token but are no longer a user)"
    )]
    YouDoNotExist,

    #[error(transparent)]
    ClientError {
        #[from]
        error: ClientError,
    },
}

#[derive(Debug, Error)]
pub enum CurrentUserDisplayNameUpdateError {
    #[error(
        "the current user does not exist \
        (perhaps you have a valid token but are no longer a user)"
    )]
    YouDoNotExist,

    #[error("the provided display name is taken")]
    DisplayNameAlreadyExists,

    #[error(transparent)]
    ClientError {
        #[from]
        error: ClientError,
    },
}



async fn get_current_user_information<C>(client: &C) -> ClientResult<UserInfo, CurrentUserReadError>
where
    C: AuthenticatedHttpClient,
{
    let response = client
        .get_request_builder()
        .endpoint_url("/users/me")
        .send_authenticated()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        let current_user_info = response.json::<UserInfoResponse>().await?;

        Ok(current_user_info.user)
    } else if response_status == StatusCode::NOT_FOUND {
        let user_error_reason = response.users_error_reason().await?;

        match user_error_reason {
            UsersErrorReason::UserNotFound => Err(CurrentUserReadError::YouDoNotExist),
            _ => handle_unexpected_error_reason!(user_error_reason, response_status),
        }
    } else {
        handle_uncaught_status_code!(response_status);
    }
}


async fn get_current_user_roles<C>(client: &C) -> ClientResult<RoleSet, CurrentUserReadError>
where
    C: AuthenticatedHttpClient,
{
    let response = client
        .get_request_builder()
        .endpoint_url("/users/me/roles")
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
    } else if response_status == StatusCode::NOT_FOUND {
        let user_error_reason = response.users_error_reason().await?;

        match user_error_reason {
            UsersErrorReason::UserNotFound => Err(CurrentUserReadError::YouDoNotExist),
            _ => handle_unexpected_error_reason!(user_error_reason, response_status),
        }
    } else {
        handle_uncaught_status_code!(response_status);
    }
}


async fn get_current_user_effective_permissions<C>(
    client: &C,
) -> ClientResult<PermissionSet, CurrentUserReadError>
where
    C: AuthenticatedHttpClient,
{
    let response = client
        .get_request_builder()
        .endpoint_url("/users/me/permissions")
        .send_authenticated()
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
            UsersErrorReason::UserNotFound => Err(CurrentUserReadError::YouDoNotExist),
            _ => handle_unexpected_error_reason!(user_error_reason, response_status),
        }
    } else {
        handle_uncaught_status_code!(response_status);
    }
}



pub struct CurrentUserDisplayNameUpdate {
    pub new_display_name: String,
}


async fn update_current_user_display_name<C>(
    client: &C,
    display_name_update: CurrentUserDisplayNameUpdate,
) -> ClientResult<UserInfo, CurrentUserDisplayNameUpdateError>
where
    C: AuthenticatedHttpClient,
{
    let response = client
        .patch_request_builder()
        .endpoint_url("/users/me/display_name")
        .json(&UserDisplayNameChangeRequest {
            new_display_name: display_name_update.new_display_name,
        })
        .send_authenticated()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        let change_response = response.json::<UserDisplayNameChangeResponse>().await?;

        Ok(change_response.user)
    } else if response_status == StatusCode::CONFLICT {
        let user_error_reason = response.users_error_reason().await?;

        match user_error_reason {
            UsersErrorReason::DisplayNameAlreadyExists => {
                Err(CurrentUserDisplayNameUpdateError::DisplayNameAlreadyExists)
            }
            _ => handle_unexpected_error_reason!(user_error_reason, response_status),
        }
    } else if response_status == StatusCode::NOT_FOUND {
        let user_error_reason = response.users_error_reason().await?;

        match user_error_reason {
            UsersErrorReason::UserNotFound => Err(CurrentUserDisplayNameUpdateError::YouDoNotExist),
            _ => handle_unexpected_error_reason!(user_error_reason, response_status),
        }
    } else {
        handle_uncaught_status_code!(response_status);
    }
}



pub struct CurrentUserApi<'c, C>
where
    C: Client + AuthenticatedHttpClient,
{
    client: &'c C,
}

impl<'c, C> CurrentUserApi<'c, C>
where
    C: Client + AuthenticatedHttpClient,
{
    pub(crate) const fn new(client: &'c C) -> Self {
        Self { client }
    }

    pub async fn get_current_user_information(&self) -> Result<UserInfo, CurrentUserReadError> {
        get_current_user_information(self.client).await
    }

    pub async fn get_current_user_roles(&self) -> Result<RoleSet, CurrentUserReadError> {
        get_current_user_roles(self.client).await
    }

    pub async fn get_current_user_effective_permissions(
        &self,
    ) -> Result<PermissionSet, CurrentUserReadError> {
        get_current_user_effective_permissions(self.client).await
    }

    pub async fn update_current_user_display_name(
        &self,
        display_name_update: CurrentUserDisplayNameUpdate,
    ) -> Result<UserInfo, CurrentUserDisplayNameUpdateError> {
        update_current_user_display_name(self.client, display_name_update).await
    }
}
