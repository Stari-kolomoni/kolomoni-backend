use chrono::{DateTime, Utc};
use kolomoni_core::api_models::{
    UserDisplayNameChangeRequest,
    UserDisplayNameChangeResponse,
    UserInfo,
    UserInfoResponse,
    UserPermissionsResponse,
    UserRolesResponse,
    UsersErrorReason,
};
use kolomoni_core::datetime::{format_utc_datetime_for_http_header, parse_http_datetime_as_utc};
use kolomoni_core::permissions::PermissionSet;
use kolomoni_core::roles::RoleSet;
use reqwest::header::HeaderValue;
use reqwest::{header, StatusCode};
use thiserror::Error;

use crate::api::users::ConditionalUserInfo;
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



/// The user information will only be returned
/// if the user has been modified since the provided datetime.
fn get_current_user_information_request_if_modified_since<'c, C>(
    client: &'c C,
    if_modified_since: &DateTime<Utc>,
) -> BoundTypedRequest<'c, C, ConditionalUserInfo, CurrentUserReadError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let current_user = response.into_json_body::<UserInfoResponse>().await?;

            Ok(ConditionalUserInfo::Modified {
                user: current_user.user,
            })
        } else if status == StatusCode::NOT_FOUND {
            let users_error_reason = response.users_error_reason().await?;

            match users_error_reason {
                UsersErrorReason::UserNotFound => Err(CurrentUserReadError::YouDoNotExist),
                _ => Err(unexpected_error_reason(
                    status,
                    users_error_reason,
                )),
            }
        } else if status == StatusCode::NOT_MODIFIED {
            let Some(last_modified_header) = response.headers().get(header::LAST_MODIFIED) else {
                return Err(RequestError::unexpected_response(
                    status,
                    "Last-Modified header is missing from the HTTP 304 Not Modified response \
                        to a conditional current user query request.",
                )
                .into());
            };

            let last_modified_at_str = last_modified_header.to_str()
                .map_err(|_| RequestError::unexpected_response(
                    status,
                    "Last-Modified header contains more than just visible ASCII characters, which is unexpected, considering it should be a date."
                ))?;

            let last_modified_at =
                parse_http_datetime_as_utc(last_modified_at_str).map_err(|_| {
                    RequestError::unexpected_response(
                        status,
                        "Last-Modified header contains an invalid datetime.",
                    )
                })?;

            Ok(ConditionalUserInfo::Unmodified { last_modified_at })
        } else {
            Err(unexpected_response(response).await)
        }
    };

    let formatted_if_modified_since = format_utc_datetime_for_http_header(if_modified_since);
    let if_modified_since_header_value = HeaderValue::from_str(&formatted_if_modified_since)
        // PANIC SAFETY: `formatted_if_modified_since` never contains any invisible ASCII characters,
        // because it's a simple datetime formatter that conforms to RFC 5322.
        .expect("creating a HeaderValue from a RFC 5322-formatted datetime should not fail");

    client
        .get()
        .endpoint_url("/users/me")
        .header(
            header::IF_MODIFIED_SINCE,
            if_modified_since_header_value,
        )
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
    fn get_current_user_information_if_modified_since(
        &'c self,
        if_modified_since: &DateTime<Utc>,
    ) -> BoundTypedRequest<'c, C, ConditionalUserInfo, CurrentUserReadError> {
        get_current_user_information_request_if_modified_since(self.client(), if_modified_since)
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
