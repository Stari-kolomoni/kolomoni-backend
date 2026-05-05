use crate::client::errors::RequestError;


pub(crate) mod raw;
mod typed;
pub use raw::RawResponse;
pub use typed::{IntoTypedResponse, TypedResponse};


pub trait ResponseValueError {
    /// All response value error types must have a variant that
    /// stores a potential [`RequestError`], which contains most general
    /// errors that a request might hit. This function will be
    /// called when that error happens in order to obtain `Self`,
    /// since in those cases this error variant will not be
    /// constructed by the custom parser closure that each endpoint provides.
    fn from_request_error(error: RequestError) -> Self;
}
