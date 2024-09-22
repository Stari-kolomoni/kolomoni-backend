//! API definitions and annotations for Stari Kolomoni.

use actix_utils::future::{self, Ready};
use actix_web::{http::header, web, FromRequest, HttpRequest, Scope};
use chrono::{DateTime, SubsecRound, Utc};

use self::v1::v1_api_router;

pub mod errors;
pub mod macros;
pub mod openapi;
pub mod traits;
pub mod v1;


// TODO document
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum OptionalIfModifiedSince {
    Unspecified,
    Specified(DateTime<Utc>),
}

impl OptionalIfModifiedSince {
    #[inline]
    fn new_unspecified() -> Self {
        Self::Unspecified
    }

    #[inline]
    fn new_specified(date_time: DateTime<Utc>) -> Self {
        Self::Specified(date_time.trunc_subsecs(0))
    }

    #[inline]
    pub fn enabled_and_has_not_changed_since(
        &self,
        real_last_modification_time: &DateTime<Utc>,
    ) -> bool {
        match self {
            OptionalIfModifiedSince::Unspecified => false,
            OptionalIfModifiedSince::Specified(user_provided_conditional_time) => {
                let user_provided_conditional_time_no_frac =
                    user_provided_conditional_time.trunc_subsecs(0);

                let real_modification_time_no_frac = real_last_modification_time.trunc_subsecs(0);

                user_provided_conditional_time_no_frac >= real_modification_time_no_frac
            }
        }
    }
}

impl FromRequest for OptionalIfModifiedSince {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        if let Some(if_modified_header_value) = req.headers().get(header::IF_MODIFIED_SINCE) {
            let Ok(if_modified_header_value) = if_modified_header_value.to_str() else {
                return future::err(actix_web::error::ParseError::Header.into());
            };

            let Ok(parsed_date_time) = httpdate::parse_http_date(if_modified_header_value) else {
                return future::err(actix_web::error::ParseError::Header.into());
            };

            let utc_time: DateTime<Utc> = parsed_date_time.into();

            future::ok(Self::new_specified(utc_time))
        } else {
            future::ok(Self::new_unspecified())
        }
    }
}



/// Router for the entire public API.
///
/// Lives under the `/api` path and is made up of `/v1` and its sub-routes.
#[rustfmt::skip]
pub fn api_router() -> Scope {
    web::scope("/api")
        .service(v1_api_router())
}
