//! This crate contains all configuration-related code, including
//! the full configuration structure, as well as methods needed to load
//! and validate it.
//!
//! Your starting point should probably be [`Configuration::load_from_default_path`].
//!
//!
//! # Internals
//! The entire configuration structure is based on the concept of
//! unresolved and resolved ("validated") configuration structures.
//!
//! Internally, we load the configuration as a different type (e.g. [`UnresolvedConfiguration`]).
//! We then then call its [`resolve`] and other similar methods, validating
//! the contents and turning itself into [`Configuration`], the public, resolved structure.
//!
//! This way we can implement any additional run-time validation on the configuration.
//! Note that the resolving methods ([`resolve`], [`try_resolve`], [`resolve_with_context`],
//! and [`try_resolve_with_context`]) **must all be side-effect free**.
//! If you need to e.g. create a directory if it is missing based on a configuration field,
//! do that in a separate method.
//!
//!
//! [`resolve`]: crate::traits::Resolve::resolve
//! [`try_resolve`]: crate::traits::TryResolve::try_resolve
//! [`resolve_with_context`]: crate::traits::ResolveWithContext::resolve_with_context
//! [`try_resolve_with_context`]: crate::traits::TryResolveWithContext::try_resolve_with_context


mod error;
mod structure;
mod traits;
mod utilities;

pub use error::*;
pub use structure::*;
