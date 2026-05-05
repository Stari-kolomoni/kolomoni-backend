//! A client for Stari Kolomoni implementing **(!) a subset of its API (!)**.

use std::{future::Future, pin::Pin};

pub mod api;
pub mod authentication;
pub mod client;
pub(crate) mod parsing;
pub mod request;
pub mod response;
pub mod server;

pub(crate) type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
