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

use crate::client::errors::RequestError;
use crate::client::KolomoniHttpClient;
use crate::parsing::{unexpected_error_reason, unexpected_response};
use crate::request::typed::{BoundTypedRequest, IntoBoundTypedRequest};
use crate::request::ToRequestBuilder;
use crate::response::raw::RawResponse;
use crate::response::ResponseValueError;


#[derive(Debug, Error)]
pub enum CurrentUserReadError {
    #[error(
        "the current user does not exist \
        (perhaps you have a valid token but are no longer a user)"
    )]
    YouDoNotExist,

    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for CurrentUserReadError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
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
    RequestError(#[from] RequestError),
}

impl ResponseValueError for CurrentUserDisplayNameUpdateError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}


fn get_current_user_information_request<'c, C>(
    client: &'c C,
) -> BoundTypedRequest<'c, C, UserInfo, CurrentUserReadError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let current_user = response.into_json_body::<UserInfoResponse>().await?;

            Ok(current_user.user)
        } else if status == StatusCode::NOT_FOUND {
            let users_error_reason = response.users_error_reason().await?;

            match users_error_reason {
                UsersErrorReason::UserNotFound => Err(CurrentUserReadError::YouDoNotExist),
                _ => Err(unexpected_error_reason(
                    status,
                    users_error_reason,
                )),
            }
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .get()
        .endpoint_url("/users/me")
        .build_request()
        .into_bound_typed_request(response_parser)
}

fn get_current_user_roles_request<'c, C>(
    client: &'c C,
) -> BoundTypedRequest<'c, C, RoleSet, CurrentUserReadError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let user_roles_response = response.into_json_body::<UserRolesResponse>().await?;

            let Ok(user_role_set) = RoleSet::try_from_role_names(user_roles_response.role_names)
            else {
                return Err(RequestError::unexpected_response(
                    status,
                    "server responded with a role set that contains at least one invalid role",
                )
                .into());
            };

            Ok(user_role_set)
        } else if status == StatusCode::NOT_FOUND {
            let users_error_reason = response.users_error_reason().await?;

            match users_error_reason {
                UsersErrorReason::UserNotFound => Err(CurrentUserReadError::YouDoNotExist),
                _ => Err(unexpected_error_reason(
                    status,
                    users_error_reason,
                )),
            }
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .get()
        .endpoint_url("/users/me/roles")
        .build_request()
        .into_bound_typed_request(response_parser)
}


fn get_current_user_effective_permissions_request<'c, C>(
    client: &'c C,
) -> BoundTypedRequest<'c, C, PermissionSet, CurrentUserReadError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let user_permissions_response =
                response.into_json_body::<UserPermissionsResponse>().await?;

            let Ok(user_permissions_set) =
                PermissionSet::try_from_permission_names(user_permissions_response.permissions)
            else {
                return Err(RequestError::unexpected_response(status, "server responded with a permission set that contains at least one invalid permission",).into());
            };

            Ok(user_permissions_set)
        } else if status == StatusCode::NOT_FOUND {
            let users_error_reason = response.users_error_reason().await?;

            match users_error_reason {
                UsersErrorReason::UserNotFound => Err(CurrentUserReadError::YouDoNotExist),
                _ => Err(unexpected_error_reason(
                    status,
                    users_error_reason,
                )),
            }
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .get()
        .endpoint_url("/users/me/permissions")
        .build_request()
        .into_bound_typed_request(response_parser)
}



pub struct CurrentUserDisplayNameUpdate {
    pub new_display_name: String,
}

fn update_current_user_display_name_request<'c, C>(
    client: &'c C,
    update: CurrentUserDisplayNameUpdate,
) -> BoundTypedRequest<'c, C, UserInfo, CurrentUserDisplayNameUpdateError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let user_change_response = response
                .into_json_body::<UserDisplayNameChangeResponse>()
                .await?;

            Ok(user_change_response.user)
        } else if status == StatusCode::CONFLICT {
            let users_error_reason = response.users_error_reason().await?;

            match users_error_reason {
                UsersErrorReason::DisplayNameAlreadyExists => {
                    Err(CurrentUserDisplayNameUpdateError::DisplayNameAlreadyExists)
                }
                _ => Err(unexpected_error_reason(
                    status,
                    users_error_reason,
                )),
            }
        } else if status == StatusCode::NOT_FOUND {
            let users_error_reason = response.users_error_reason().await?;

            match users_error_reason {
                UsersErrorReason::UserNotFound => {
                    Err(CurrentUserDisplayNameUpdateError::YouDoNotExist)
                }
                _ => Err(unexpected_error_reason(
                    status,
                    users_error_reason,
                )),
            }
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .patch()
        .endpoint_url("/users/me/display_name")
        .json(&UserDisplayNameChangeRequest {
            new_display_name: update.new_display_name,
        })
        .build_request()
        .into_bound_typed_request(response_parser)
}


pub trait CurrentUserEndpoints<'c, C>
where
    C: KolomoniHttpClient,
{
    fn client(&'c self) -> &'c C;

    #[inline(always)]
    fn get_current_user_information(
        &'c self,
    ) -> BoundTypedRequest<'c, C, UserInfo, CurrentUserReadError> {
        get_current_user_information_request(self.client())
    }

    #[inline(always)]
    fn get_current_user_roles(&'c self) -> BoundTypedRequest<'c, C, RoleSet, CurrentUserReadError> {
        get_current_user_roles_request(self.client())
    }

    #[inline(always)]
    fn get_current_user_effective_permissions(
        &'c self,
    ) -> BoundTypedRequest<'c, C, PermissionSet, CurrentUserReadError> {
        get_current_user_effective_permissions_request(self.client())
    }

    #[inline(always)]
    fn update_current_user_display_name(
        &'c self,
        update: CurrentUserDisplayNameUpdate,
    ) -> BoundTypedRequest<'c, C, UserInfo, CurrentUserDisplayNameUpdateError> {
        update_current_user_display_name_request(self.client(), update)
    }
}


pub struct CurrentUserApi<'c, C>
where
    C: KolomoniHttpClient,
{
    client: &'c C,
}

impl<'c, C> CurrentUserApi<'c, C>
where
    C: KolomoniHttpClient,
{
    #[inline(always)]
    pub(crate) const fn new(client: &'c C) -> Self {
        Self { client }
    }
}

impl<'c, C> CurrentUserEndpoints<'c, C> for CurrentUserApi<'c, C>
where
    C: KolomoniHttpClient,
{
    fn client(&'c self) -> &'c C {
        self.client
    }
}
