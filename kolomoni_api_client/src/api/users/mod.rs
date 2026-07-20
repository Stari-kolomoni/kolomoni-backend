use chrono::{DateTime, Utc};
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

use crate::api::EndpointGroup;
use crate::client::errors::RequestError;
use crate::client::KolomoniHttpClient;
use crate::parsing::{
    err_if_invalid_uuid,
    err_if_missing_permissions,
    unexpected_error_reason,
    unexpected_response,
};
use crate::request::typed::{BoundTypedRequest, IntoBoundTypedRequest};
use crate::request::ToRequestBuilder;
use crate::response::raw::RawResponse;
use crate::response::ResponseValueError;
pub mod current;

/// Returned from conditional requests for user information
/// based on the `If-Modified-Since` header:
/// - If `Self::Unmodified`,
///   the user has not been modified since the provided datetime
///   and the response does not contain the user information,
///   just the `Last-Modified` header.
/// - If `Self::Modified`,
///   the user has been modified since the provided datetime
///   and therefore the response contains the full user information.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ConditionalUserInfo {
    Unmodified { last_modified_at: DateTime<Utc> },
    Modified { user: UserInfo },
}


#[derive(Debug, Error)]
pub enum UserListError {
    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for UserListError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}


fn get_all_registered_users_request<'c, C>(
    client: &'c C,
) -> BoundTypedRequest<'c, C, Vec<UserInfo>, UserListError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let registered_users_response = response
                .into_json_body::<RegisteredUsersListResponse>()
                .await?;

            Ok(registered_users_response.users)
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .get()
        .endpoint_url("/users")
        .build_request()
        .into_bound_typed_request(response_parser)
}


#[derive(Debug, Error)]
pub enum UserDataFetchError {
    #[error("a user with the specified ID does not exist")]
    NotFound,

    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for UserDataFetchError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}

fn get_user_information_by_user_id_request<'c, C>(
    client: &'c C,
    user_id: UserId,
) -> BoundTypedRequest<'c, C, UserInfo, UserDataFetchError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let user_info_response = response.into_json_body::<UserInfoResponse>().await?;

            Ok(user_info_response.user)
        } else if status == StatusCode::NOT_FOUND {
            let users_error_response = response.users_error_reason().await?;

            match users_error_response {
                UsersErrorReason::UserNotFound => Err(UserDataFetchError::NotFound),
                _ => Err(unexpected_error_reason(
                    status,
                    users_error_response,
                )),
            }
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .get()
        .endpoint_url(format!("/users/{}", user_id))
        .build_request()
        .into_bound_typed_request(response_parser)
}


fn get_user_roles_by_user_id_request<'c, C>(
    client: &'c C,
    user_id: UserId,
) -> BoundTypedRequest<'c, C, RoleSet, UserDataFetchError>
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
            let users_error_response = response.users_error_reason().await?;

            match users_error_response {
                UsersErrorReason::UserNotFound => Err(UserDataFetchError::NotFound),
                _ => Err(unexpected_error_reason(
                    status,
                    users_error_response,
                )),
            }
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .get()
        .endpoint_url(format!("/users/{}/roles", user_id))
        .build_request()
        .into_bound_typed_request(response_parser)
}


fn get_user_effective_permissions_by_user_id_request<'c, C>(
    client: &'c C,
    user_id: UserId,
) -> BoundTypedRequest<'c, C, PermissionSet, UserDataFetchError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let user_permissions_response =
                response.into_json_body::<UserPermissionsResponse>().await?;

            let Ok(user_permission_set) =
                PermissionSet::try_from_permission_names(user_permissions_response.permissions)
            else {
                return Err(
                    RequestError::unexpected_response(
                        status,
                        "server responded with a permission set that contains at least one invalid permission"
                    ).into()
                );
            };

            Ok(user_permission_set)
        } else if status == StatusCode::NOT_FOUND {
            let users_error_response = response.users_error_reason().await?;

            match users_error_response {
                UsersErrorReason::UserNotFound => Err(UserDataFetchError::NotFound),
                _ => Err(unexpected_error_reason(
                    status,
                    users_error_response,
                )),
            }
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .get()
        .endpoint_url(format!("/users/{}/permissions", user_id))
        .build_request()
        .into_bound_typed_request(response_parser)
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
    RequestError(#[from] RequestError),
}

impl ResponseValueError for UserDisplayNameUpdateError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}


fn update_user_display_name_by_user_id_request<'c, C, S>(
    client: &'c C,
    user_id: UserId,
    new_display_name: S,
) -> BoundTypedRequest<'c, C, UserInfo, UserDisplayNameUpdateError>
where
    C: KolomoniHttpClient,
    S: Into<String>,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let user_display_name_change = response
                .into_json_body::<UserDisplayNameChangeResponse>()
                .await?;

            Ok(user_display_name_change.user)
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            if let ErrorReason::Users(users_error_reason) = &error_reason {
                if users_error_reason == &UsersErrorReason::CannotModifyYourOwnAccount {
                    return Err(UserDisplayNameUpdateError::CannotModifyYourself);
                }
            }

            err_if_missing_permissions::<UserDisplayNameUpdateError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else if status == StatusCode::NOT_FOUND {
            let users_error_reason = response.users_error_reason().await?;

            match users_error_reason {
                UsersErrorReason::UserNotFound => Err(UserDisplayNameUpdateError::NotFound),
                _ => Err(unexpected_error_reason(
                    status,
                    users_error_reason,
                )),
            }
        } else if status == StatusCode::CONFLICT {
            let users_error_reason = response.users_error_reason().await?;

            match users_error_reason {
                UsersErrorReason::DisplayNameAlreadyExists => {
                    Err(UserDisplayNameUpdateError::DisplayNameAlreadyExists)
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
        .endpoint_url(format!("/users/{}/display_name", user_id))
        .json(&UserDisplayNameChangeRequest {
            new_display_name: new_display_name.into(),
        })
        .build_request()
        .into_bound_typed_request(response_parser)
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
    RequestError(#[from] RequestError),
}

impl ResponseValueError for UserRolesAddError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}



fn add_roles_to_user_by_user_id_request<'c, C>(
    client: &'c C,
    user_id: UserId,
    roles_to_add: RoleSet,
) -> BoundTypedRequest<'c, C, RoleSet, UserRolesAddError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let updated_roles_response = response.into_json_body::<UserRolesResponse>().await?;

            let Ok(updated_role_set) =
                RoleSet::try_from_role_names(updated_roles_response.role_names)
            else {
                return Err(RequestError::unexpected_response(
                    status,
                    "server responded with a role set that contains at least one invalid role",
                )
                .into());
            };

            Ok(updated_role_set)
        } else if status == StatusCode::BAD_REQUEST {
            let error_reason = response.error_reason().await?;

            #[allow(clippy::collapsible_match)]
            if let ErrorReason::Users(users_error_reason) = &error_reason {
                if let UsersErrorReason::InvalidRoleName { .. } = users_error_reason {
                    return Err(RequestError::unexpected_response(
                        status,
                        "server rejected one of our role names as invalid; \
                            this may indicate the client's available role set is out of date",
                    )
                    .into());
                };
            };

            err_if_invalid_uuid::<UserRolesAddError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            if let ErrorReason::Users(users_error_reason) = &error_reason {
                match users_error_reason {
                    UsersErrorReason::CannotModifyYourOwnAccount => {
                        return Err(UserRolesAddError::CannotModifyYourself);
                    }
                    UsersErrorReason::UnableToGiveOutUnownedRole { .. } => {
                        return Err(UserRolesAddError::CannotGiveOutRolesYouDontHave);
                    }
                    _ => {}
                }
            }

            err_if_missing_permissions::<UserRolesAddError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else if status == StatusCode::NOT_FOUND {
            let users_error_reason = response.users_error_reason().await?;

            match users_error_reason {
                UsersErrorReason::UserNotFound => Err(UserRolesAddError::UserNotFound),
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
        .post()
        .endpoint_url(format!("/users/{}/roles", user_id))
        .json(&UserRoleAddRequest {
            roles_to_add: roles_to_add.role_names(),
        })
        .build_request()
        .into_bound_typed_request(response_parser)
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
    RequestError(#[from] RequestError),
}

impl ResponseValueError for UserRolesRemoveError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}

fn remove_roles_from_user_by_user_id_request<'c, C>(
    client: &'c C,
    user_id: UserId,
    roles_to_remove: RoleSet,
) -> BoundTypedRequest<'c, C, RoleSet, UserRolesRemoveError>
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
        } else if status == StatusCode::BAD_REQUEST {
            let error_reason = response.error_reason().await?;

            if let ErrorReason::Users(users_error_reason) = &error_reason {
                if matches!(
                    users_error_reason,
                    UsersErrorReason::InvalidRoleName { .. }
                ) {
                    return Err(RequestError::unexpected_response(
                        status,
                        "server rejected one of our role names as invalid; \
                        this may indicate the client's available role set is out of date",
                    )
                    .into());
                }
            }

            err_if_invalid_uuid::<UserRolesRemoveError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else if status == StatusCode::FORBIDDEN {
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

            err_if_missing_permissions::<UserRolesRemoveError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else if status == StatusCode::NOT_FOUND {
            let users_error_reason = response.users_error_reason().await?;

            match users_error_reason {
                UsersErrorReason::UserNotFound => Err(UserRolesRemoveError::UserNotFound),
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
        .delete()
        .endpoint_url(format!("/users/{}/roles", user_id))
        .json(&UserRoleRemoveRequest {
            roles_to_remove: roles_to_remove.role_names(),
        })
        .build_request()
        .into_bound_typed_request(response_parser)
}


pub trait SpecificUserUnauthenticatedEndpoints<'c, C>: EndpointGroup<'c, C>
where
    C: KolomoniHttpClient,
{
    fn get_user_information_by_user_id(
        &'c self,
        user_id: UserId,
    ) -> BoundTypedRequest<'c, C, UserInfo, UserDataFetchError> {
        get_user_information_by_user_id_request(self.client(), user_id)
    }

    fn get_user_roles_by_user_id(
        &'c self,
        user_id: UserId,
    ) -> BoundTypedRequest<'c, C, RoleSet, UserDataFetchError> {
        get_user_roles_by_user_id_request(self.client(), user_id)
    }

    fn get_user_effective_permissions_by_user_id(
        &'c self,
        user_id: UserId,
    ) -> BoundTypedRequest<'c, C, PermissionSet, UserDataFetchError> {
        get_user_effective_permissions_by_user_id_request(self.client(), user_id)
    }
}

pub trait SpecificUserAuthenticatedEndpoints<'c, C>: EndpointGroup<'c, C>
where
    C: KolomoniHttpClient,
{
    fn get_all_registered_users(&'c self) -> BoundTypedRequest<'c, C, Vec<UserInfo>, UserListError> {
        get_all_registered_users_request(self.client())
    }

    fn update_user_display_name_by_user_id<U>(
        &'c self,
        user_id: UserId,
        new_display_name: U,
    ) -> BoundTypedRequest<'c, C, UserInfo, UserDisplayNameUpdateError>
    where
        U: Into<String>,
    {
        update_user_display_name_by_user_id_request(self.client(), user_id, new_display_name)
    }

    fn add_roles_to_user_by_user_id(
        &'c self,
        user_id: UserId,
        roles_to_add: RoleSet,
    ) -> BoundTypedRequest<'c, C, RoleSet, UserRolesAddError> {
        add_roles_to_user_by_user_id_request(self.client(), user_id, roles_to_add)
    }

    fn remove_roles_from_user_by_user_id(
        &'c self,
        user_id: UserId,
        roles_to_remove: RoleSet,
    ) -> BoundTypedRequest<'c, C, RoleSet, UserRolesRemoveError> {
        remove_roles_from_user_by_user_id_request(self.client(), user_id, roles_to_remove)
    }
}


pub struct SpecificUserUnauthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
    client: &'c C,
}

impl<'c, C> SpecificUserUnauthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
    pub(crate) const fn new(client: &'c C) -> Self {
        Self { client }
    }
}

impl<'c, C> EndpointGroup<'c, C> for SpecificUserUnauthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
    fn client(&'c self) -> &'c C {
        self.client
    }
}

impl<'c, C> SpecificUserUnauthenticatedEndpoints<'c, C> for SpecificUserUnauthenticatedApi<'c, C> where
    C: KolomoniHttpClient
{
}



pub struct SpecificUserAuthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
    client: &'c C,
}

impl<'c, C> SpecificUserAuthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
    pub(crate) const fn new(client: &'c C) -> Self {
        Self { client }
    }
}

impl<'c, C> EndpointGroup<'c, C> for SpecificUserAuthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
    fn client(&'c self) -> &'c C {
        self.client
    }
}

impl<'c, C> SpecificUserUnauthenticatedEndpoints<'c, C> for SpecificUserAuthenticatedApi<'c, C> where
    C: KolomoniHttpClient
{
}

impl<'c, C> SpecificUserAuthenticatedEndpoints<'c, C> for SpecificUserAuthenticatedApi<'c, C> where
    C: KolomoniHttpClient
{
}
