use std::time::Duration;

use kolomoni_configuration::ForApiDatabaseConfiguration;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};

pub mod api;
pub(crate) mod authentication;
pub(crate) mod cli;
pub mod logging;
pub(crate) mod state;

// TODO -- things to do, in rough order: --
// DONE migrate to the new database structure, which will remove and add some new endpoints
// DONE refactor actix extractors/data into a better structure
// DONE refactor non-library things out of this crate into kolomoni_core (including API request/response models?)
// TODO review documentation, especially top-level crate docs (+ check for cargo doc warnings)
// TODO refactor how state is updated locally, so it can be more general than just for the search crate
// TODO rework search crate with either a deep-dive into tantivy or by removing tantivy and using manual similarity metrics
// TODO rework the kolomoni_sample_data to be rust, and to ingest the Google Sheets document for seeding data
// TODO migrate tests to new database structure
// TODO for clarity, create two directories: `crates` and `binaries`, where workspaces crates will be categorized
//      (e.g. `kolomoni` + `kolomoni_openapi` can go in `binaries`)
// TODO review CI
// TODO review makefile
// TODO review unused dependencies
