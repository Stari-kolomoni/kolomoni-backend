//! One special thing regarding the API method naming scheme:
//! for each API endpoint, there are *two* methods. For example,
//! looking at [`users::current::CurrentUserApi`]:
//! - [`CurrentUserApi::get_current_user_information_response`] returns a
//!   [`TypedServerResponse`]`<`[`UserInfo`]`,`[`CurrentUserReadError`]`, _>`.
//!   This huge type represents a TODO
// TODO
//!
//!
//! [`CurrentUserApi`]: [`users::current::CurrentUserApi`]
//! [`CurrentUserApi::get_current_user_information_response`]: [`users::current::CurrentUserApi::get_current_user_information_response`]
//! [`RawServerResponse`]: [`crate::response::RawServerResponse`]
//! [`ResponseValueResult`]: [`crate::response::ResponseValueResult`]
//! [`TypedServerResponse`]: [`crate::response::TypedServerResponse`]
//! [`UserInfo`]: [`kolomoni_core::api_models::users::UserInfo`]
//! [`CurrentUserReadError`]: [`crate::api::users::current::CurrentUserReadError`]

use crate::client::KolomoniHttpClient;

pub mod auth;
pub mod dictionary;
pub mod health;
pub mod users;


pub trait EndpointGroup<'c, C>
where
    C: KolomoniHttpClient,
{
    fn client(&'c self) -> &'c C;
}
