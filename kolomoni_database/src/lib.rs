//! This crate takes care of all of the database (PostgreSQL) interactions.
//!
//! Users of this crate should primarily operate on the [`entities`][crate::entities] module.
//!
//! # Crate structure
//! At the top level this crate exposes the following important modules and types:
//! - The [`DatabaseConnectionPool`] handles everything regarding the PostgreSQL connection pool.
//!   The pool is initialized by calling [`DatabaseConnectionPool::connect`], where it can be configured
//!   using [`DatabaseConnectionOptions`] (connection parameters) and
//!   [`DatabaseConnectionPoolOptions`] (pool parameters).
//! - [`DatabaseConnection`] and [`DatabaseTransaction`]) represent a single pool connection
//!   and a transaction for some connection, respectively.
//! - The [`entities`] module, which contains *all* of the abstractions required to create, read, update,
//!   and delete any of the database models. If something needs to interact with the database,
//!   excluding the migration system, it should go through this.
//!
//! ## The [`entities`] module
//! The entities module contains sub-modules that each handle a different section of the database
//! (usually a table each). Each module contains the following:
//! - `models.rs` contains the related structs that describe all of the internal and external models.
//!   Inside it, there are generally at most four sub-modules:
//!   - `internal_weak` (`pub(crate)`), which contains "weak"(ly typed) internal models.
//!     The phrasing "weak" here refers to the fact that, usually, these models
//!     contain data that is not fully validated or strongly-typed (mostly due to limitations in [`sqlx`]).
//!     These models must therefore eventually be converted into their normal — "strong", if you will —
//!     internal model counterparts. This is usually done via the [`TryIntoStronglyTypedInternalModel`] impl
//!     for that type.
//!   - `internal_insert_only` (`pub(crate)`), which contains "insert-only" internal models.
//!     This phrasing refers to models that are used only beside the queries themselves when inserting rows.
//!     To elaborate: when doing an `INSERT`, we often include a `RETURNING` clause, because we want the data to round-trip.
//!     Therefore, we need models that hold the contents of the insert-returned queries.
//!   - `internal` (`pub(crate)`), which contains the usual (fully strongly-typed) internal models.
//!     The phrasing "internal" here refers to the fact these models are not visible outside of the crate,
//!     and they are used only as direct or indirect return types for compile-time checked SQL queries themselves.
//!     Most of these types implement [`IntoExternalModel`], allowing us to convert them into associated external models.
//!   - `external` (`pub` re-exported at root of `models.rs`), which contains the external models.
//!     In contrast with internal models, these external models are considered public API for this crate and are therefore
//!     slightly more nested/differently-structured in their implementation to avoid repeating code (many of the attributes
//!     will need to be accessed with methods).
//! - `mod.rs` `pub`-re-exports all of the sub-modules that were just described.
//!
//! </br>
//!
//! # Traits
//! As briefly mentioned in the section above, this crate introduces the following model conversion-related traits:
//! - [`TryIntoStronglyTypedInternalModel`],
//! - [`IntoExternalModel`],
//! - [`TryIntoExternalModel`], and
//! - [`IntoInternalModel`].
//!
//! None of these traits are public (they are `pub(crate)`). See documentation on individual traits for explanations.
//!
//!
//! </br>
//!
//! # Other internal details
//! Under the hood, we use mostly compile-time checked [`sqlx`] queries, with some exceptions
//! for complicated queries (e.g. when the fields that need to be updated vary depending
//! on the function arguments).

use std::borrow::Cow;

use thiserror::Error;

pub(crate) mod macros;

pub mod entities;

mod connection;
pub use connection::*;


/// An error that occurs while interacting with the database or performing other database-related operations.
#[derive(Debug, Error)]
pub enum QueryError {
    /// A query execution error. This generally indicates the SQL query itself failed.
    #[error("sqlx error")]
    SqlxError {
        #[from]
        #[source]
        error: sqlx::Error,
    },

    /// Model error. This indicates the query likely executed correctly,
    /// but an error occurred while parsing the result of the query into some model.
    ///
    /// The `reason` field should explain what went wrong in more detail.
    #[error("model error: {}", .reason)]
    ModelError { reason: Cow<'static, str> },

    /// Critical database error. This error variant indicates that
    /// an invariant of the database — or more likely our schema — has been violated.
    ///
    /// An example of this would be that a deletion of a word by its UUID affecting more than one row,
    /// which should be impossible, because UUIDs were (are) primary keys when this database layer was
    /// designed and implemented. These kinds of checks should help us preserve some basic database invariants,
    /// catching the errors if we were to accidentally modify some important aspect of the schema in the future.
    ///
    /// **As such, treat this error variant as a critical error: if you encounter this error,
    /// something has gone terribly wrong and you should investigate.**
    #[error("database inconsistency: {}", .reason)]
    DatabaseInconsistencyError { reason: Cow<'static, str> },
}

impl QueryError {
    pub(crate) fn model_error<R>(reason: R) -> Self
    where
        R: Into<Cow<'static, str>>,
    {
        Self::ModelError {
            reason: reason.into(),
        }
    }

    pub(crate) fn database_inconsistency<R>(reason: R) -> Self
    where
        R: Into<Cow<'static, str>>,
    {
        Self::DatabaseInconsistencyError {
            reason: reason.into(),
        }
    }
}


/// A specialized [`Result`] type for this crate, defaulting the error type to [`QueryError`].
pub type QueryResult<R, E = QueryError> = Result<R, E>;


/// Converts a weak internal model into its strong internal counterpart
/// (see the top-level documentation for rationale).
pub(crate) trait TryIntoStronglyTypedInternalModel {
    type InternalModel;
    type Error;

    fn try_into_strongly_typed_internal_model(self) -> Result<Self::InternalModel, Self::Error>;
}



/// Infallibly converts a normal (strong) internal model into its external model counterpart
/// (see the top-level documentation for rationale).
pub(crate) trait IntoExternalModel {
    type ExternalModel;

    fn into_external_model(self) -> Self::ExternalModel;
}

/// Fallibly converts a normal (strong) internal model into its external model counterpart
/// (see the top-level documentation for rationale).
pub(crate) trait TryIntoExternalModel {
    type ExternalModel;
    type Error;

    fn try_into_external_model(self) -> Result<Self::ExternalModel, Self::Error>;
}


/// Infallibly converts an external model back into its strong internal model counterpart
/// (see the top-level documentation for rationale).
///
/// This operation is very rare.
pub(crate) trait IntoInternalModel {
    type InternalModel;

    fn into_internal_model(self) -> Self::InternalModel;
}
