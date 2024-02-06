//! This crate contains all configuration-relevant code, including
//! the full configuration structure as well as methods needed to load
//! and validate it.
//!
//! Your starting point should probably be [`Configuration::load_from_default_path`].
//!
//! # Internals
//! The entire configuration structure is based on the concept of
//! unvalidated ("unresolved") and validated configuration structures.
//!
//! For example, even though we're interacting with [`Configuration`],
//! it internally attempts to load the configuration file and deserialize it
//! into the [`UnresolvedConfiguration`] structure.
//! It will then call its `resolve`
//! method, which will recursively turn it
//! (and potentially its fields) into validated ("resolved") versions.
//!
//! The output will then be the [`Configuration`]. This way we can implement any
//! additional configuration validation in [`resolve`][traits::ResolvableConfiguration::resolve],
//! e.g. raising an error if some specified file path doesn't actually exist.

#![allow(rustdoc::private_intra_doc_links)]

mod structure;
mod traits;
mod utilities;

pub use structure::*;
