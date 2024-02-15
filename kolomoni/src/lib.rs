//! Stari Kolomoni backend API project.
//!
//! # Workspace structure
//! - [`kolomoni`][crate] *(this crate)* --- provides the entire API surface,
//!   with [`actix_web`][actix_web] as the server software.
//! - [`kolomoni_auth`][kolomoni_auth] --- contains authentication, role,
//!   and JSON Web Token-related code.
//! - [`kolomoni_configuration`][kolomoni_configuration] --- contains the entire configuration schema,
//!   including code to load it and validate it.
//! - [`kolomoni_database`][kolomoni_database] --- handles the entire PostgreSQL
//!   database interaction (with [SeaORM][sea_orm] as an ORM layer).
//! - [`kolomoni_migrations`][kolomoni_migrations] --- contains database migrations from which the
//!   entire schema is autogenerated in [`kolomoni_database`][kolomoni_database].
//! - [`kolomoni_openapi`](../kolomoni_openapi/index.html ) --- generates an OpenAPI schema for the entire API surface.
//!   Most annotations from which this is generated are present near each endpoint function in
//!   [`kolomoni::api::v1`][crate::api::v1], but the finishing touches are done in this crate. This crate also has a binary
//!   that serves the API schema interactively through a [RapiDoc](https://rapidocweb.com/) frontend.
//!
//!
//! # Structure of this crate
//! ```markdown
//! kolomoni/src
//! |
//! |-| api/
//! | |
//! | |-| v1/
//! | |   > Contains the entire API surface.
//! | |
//! | |-> errors.rs
//! | |   > Ways of handling errors, namely the `APIError` struct, which allows
//! | |   > you to simply return an `Err(APIError)` and have it automatically
//! | |   > return a relevant HTTP error response. Also important: `EndpointResult`.
//! | |
//! | |-> macros.rs
//! | |   > Macros to avoid repeating code, such as `impl_json_response_builder`,
//! | |   > which enables structs to automatically convert to 200 OK JSON via `into_response`.
//! | |   > Also has macros to handle authentication and require permissions.
//! | |
//! | |-> openapi.rs
//! | |   > Defines commonly-used OpenAPI / `utoipa` parameters and responses,
//! | |   > which you can then use when documenting endpoint functions with
//! | |   > the `utoipa::path` macro.
//! |
//! |-> authentication.rs
//! |   > Authentication-related code, namely an Actix extractor that
//! |   > allows us to ergonomically check for roles and permissions.
//! |
//! |-> cli.rs
//! |   > Definition of the command-line interface.
//! |
//! |-> logging.rs
//! |   > Sets up logging via the `tracing` crate.
//! |
//! |-> state.rs
//! |   > Houses the entire application state that is shared between workers.
//! |   > It contains things like the current configuration and database connection.
//! ```
//!


pub mod api;
pub mod authentication;
pub mod cli;
pub mod logging;
pub mod state;
