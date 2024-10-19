//! A client for Stari Kolomoni implementing **(!) a subset of its API (!)**.

pub(crate) mod macros;

pub mod api;
pub mod authentication;
mod clients;
pub mod errors;
pub(crate) mod request;
pub(crate) mod response;
pub use clients::*;
mod server;
pub use server::*;
