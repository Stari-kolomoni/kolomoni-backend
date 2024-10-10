use std::str::FromStr;

use actix_web::{web, Scope};
use english::english_dictionary_router;
use kolomoni_core::id::KolomoniUuidNewtype;
use slovene::slovene_dictionary_router;

use self::{
    categories::categories_router,
    // suggestions::suggested_translations_router,
    translations::translations_router,
};
use crate::api::errors::EndpointError;

pub mod categories;
pub mod english;
pub mod slovene;
// TODO
// pub mod search;
// DEPRECATED
// pub mod suggestions;
pub mod translations;



/// Given a string or a string slice (or something that implements `AsRef<str>`),
/// this function attempts to parse the string as a UUID, returning it
/// as the specified Stari Kolomoni UUID newtype, e.g. [`UserId`], [`CategoryId`], ...
///
/// # Example
/// ```rust
/// use crate::api::v1::dictionary::parse_uuid;
/// use crate::api::errors::EndpointResult;
///
/// #[actix_web::get("/{hello_world}")]
/// async fn hello_world(
///     parameters: actix_web::web::Path<(String, )>
///     // ...
/// ) -> EndpointResult {
///     // ...
///     
///     // Both flavors are valid, the turbofish syntax perhaps slightly clearer.
///     let user_id = parse_uuid::<UserId>(parameters.into_inner().0)?;
///     let user_id: UserId = parse_uuid(parameters.into_inner().0)?;
///
///     // ...
///     # todo!();
/// }
/// ```
///
///
/// [`UserId`]: kolomoni_core::id::UserId
/// [`CategoryId`]: kolomoni_core::id::CategoryId
#[inline]
pub fn parse_uuid<U>(string: impl AsRef<str>) -> Result<U, EndpointError>
where
    U: KolomoniUuidNewtype + FromStr<Err = uuid::Error>,
{
    U::from_str(string.as_ref()).map_err(|error| EndpointError::InvalidUuidFormat { error })
}


#[rustfmt::skip]
pub fn dictionary_router() -> Scope {
    web::scope("/dictionary")
        .service(slovene_dictionary_router())
        .service(english_dictionary_router())
        // DEPRECATED
        // .service(suggested_translations_router())
        .service(translations_router())
        .service(categories_router())
        // TODO
        // .service(search_router())
}
