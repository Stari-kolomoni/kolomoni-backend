<div align="center">
  <h1 align="center">Stari Kolomoni (backend)</h1>
  <h6 align="center">backend for the open fantasy translation project</h6>
</div>

<br>

This repository contains the full backend for the Stari Kolomoni open fantasy translation project as well as its Rust backend client.
See [the main page](https://github.com/Stari-kolomoni) for more information about the project.


<br>

---

# 1. Workspace structure
- `kolomoni` (main binary) defines the entire V1 API and serves the whole backend (using `actix-web`).
- `kolomoni_core` contains reusable functionality (API models and other reusable types,
  JWT token code, permission and role systems, etc.).
- `kolomoni_configuration` defines the entire configuration schema and the code to load it from disk and validate it.
- `kolomoni_cache` implements a full type-safe memory cache for database entities (this is used by our search engine
  for very fast search results).
- `kolomoni_database` handles PostgreSQL database querying and modification (using `sqlx`).
- `kolomoni_search` implements our custom search engine (based on `tantivy`).
- `kolomoni_search_core` contains shared search engine models and code for reuse.
- `kolomoni_migrations` implements a custom database migration system (see `migrations` subfolder).
- `kolomoni_migrations_core` defines reusable migration models and functionality.
- `kolomoni_migrations_macros` defines handy proc macros for embedding migrations into the binary at build time
  and for being able to elegantly write migrations in pure Rust instead of SQL (if more complex migration logic is required).
- `kolomoni_seeder` implementst a system for seeding our database with existing words, meanings, and translations
  from our previous translation system (based on spreadsheets and CSV).
- `kolomoni_api_client` implements a full-featured API client (with proper error handling for each endpoint) for our Stari Kolomoni API.
  This is particularly useful for our end-to-end tests.
- `kolomoni_openapi` assembles an OpenAPI schema for the entire API surface (it can emit it as a JSON file or serve it through the [RapiDoc](https://rapidocweb.com/) frontend).
  The final OpenAPI schema is assembled from individual annotations that are present near each endpoint function in
  `kolomoni::api::v1`.
- `kolomoni_test` contains end-to-end tests for our entire backend.
- `kolomoni_test_core` contains shared code for the end-to-end tests, wrapping the `kolomoni_api_client` crate to provide a nice test interface
  as well as providing some sample data to test on.
- `kolomoni_test_macros` defines a simple custom `#[test]` macro for setting up asynchronous tests and other features.


<br>
<br>

---

# 2. Development
This section describes how to set up this repository for local development. 


## 2.1 Requirements
Before proceeding, verify that you have the following:
- A relatively modern version of Rust (tested on `Rust 1.98`), install with [*rustup*](https://rustup.rs/).
- `PostgreSQL 15.3` or newer.
  > If you're on Windows, install or extract the portable version to somewhere on the `PATH`. There are Powershell helper scripts available in `scripts/database` for creating the database and running it while you're developing. To simplify things, you can also simply download a portable PostgreSQL archive instead of installing it. You can then, instead of adding it to `PATH`, extract its `pgsql` directory to `scripts/database` (i.e. so that the `scripts/database/pgsql/bin` directory exists).
- [`sqlx`](https://github.com/transact-rs/sqlx) CLI, installed with `cargo install --force sqlx-cli`. This will be used to manage [offline type-checked queries](https://github.com/transact-rs/sqlx/tree/main/sqlx-cli#enable-building-in-offline-mode-with-query).
- [`cargo-make`](https://github.com/sagiegurari/cargo-make), installed with `cargo install --force cargo-make`, which we'll use as our task runner.


## 2.2 Code style
> Important: *use nightly rustfmt*.

Use nightly [`rustfmt`](https://github.com/rust-lang/rustfmt) (`cargo +nightly fmt` or 
[equivalent IDE support](https://github.com/rust-lang/rustfmt#running-rustfmt-from-your-editor)) for formatting your
Rust code in this repository. Current rules can be seen in `rustfmt.toml`.


<details>
<summary>Setup for Visual Studio Code (with <code>rust-analyzer</code>)</summary>
<br>

> This configuration requires [`rust-analyzer`](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) 
> to be installed and enabled in Visual Studio Code.

If you're using Visual Studio Code, you can use something akin to the configuration below to 
enable `clippy` and `rustfmt` as described above. Add these entries into your project-local `.vscode/settings.json`,
creating the file if necessary:

```json
{
    "[rust]": {
        "editor.defaultFormatter": "rust-lang.rust-analyzer",
        "editor.formatOnSave": true
    },
    "rust-analyzer.check.overrideCommand": [
        "cargo",
        "clippy",
        "--workspace",
        "--message-format=json",
        "--all-targets",
    ],
    "rust-analyzer.rustfmt.extraArgs": [
        "+nightly"
    ],
    "rust-analyzer.cargo.features": "all"
}
```

Alongside `rust-analyzer` and this configuration, the following extensions are also recommended:
- **(highly recommended)** [EditorConfig](https://marketplace.visualstudio.com/items?itemName=EditorConfig.EditorConfig),
- *(good-to-have)* [Even Better TOML](https://marketplace.visualstudio.com/items?itemName=tamasfe.even-better-toml), and
- *(optional; highlights comments)* [Better Comments](https://marketplace.visualstudio.com/items?itemName=aaron-bond.better-comments).

For Better Comments, the following configuration might be of use (add this to `.vscode/settings.json` after installing the extension):

```json
{
    // ...
    "better-comments.tags": [
        {
            "tag": "todo",
            "color": "#77BAF5",
            "strikethrough": false,
            "underline": false,
            "backgroundColor": "transparent",
            "bold": false,
            "italic": false
        },
        {
            "tag": "debugonly",
            "color": "#c4b1e5",
            "strikethrough": false,
            "underline": false,
            "backgroundColor": "transparent",
            "bold": false,
            "italic": false
        },
        {
            "tag": "deprecated",
            "color": "#F5A867",
            "strikethrough": false,
            "underline": false,
            "backgroundColor": "transparent",
            "bold": false,
            "italic": false
        },
        {
            "tag": "fixme",
            "color": "#f26344",
            "strikethrough": false,
            "underline": false,
            "backgroundColor": "transparent",
            "bold": false,
            "italic": false
        },
    ]
    // ...
}
```

</details>



## 2.3 (Windows) Setting up the database

First off, initialize the PostgreSQL database.
If you're on *Windows*, you can use the `scripts/database/init-database.ps1` and `scripts/database/run-database.ps1` scripts for easy setup and running (or use `cargo make database:initialize` and `cargo make database:run` instead).

> **Note that this must only be used for development as it uses `--auth=trust`!**

The generated user is `kolomon` (password `kolomon`) and the database name is `stari_kolomoni`. 
The database will run on `127.0.0.1` on default port `5432`.


## 2.4 Setting up `configuration.toml`

Then, copy `data/configuration.TEMPLATE.toml` to `data/configuration.toml` and 
fill out the configuration fields (see comments in the template for explanations).


## 2.5 Building

Build the project by running `cargo build --release`. 
This will be a fresh build, so it might take a few minutes (around 2 minutes on a hard drive with a 12-core CPU at 3.6 GHz).


## 2.5 Starting the backend server for development
To start the backend server, execute `cargo make backend:run:release` (or run the `kolomoni` binary in `./target/debug`).


## Appendix

### Appendix A. About database migrations
Our database migrations are managed by ourselves (see `kolomoni_migrations`, `kolomoni_migrations_core` and `kolomoni_migrations_macros`). 

Inside the `kolomoni_migrations/migrations` directory you'll find a sequence of subdirectories, e.g. `M0001_prepare-database`, `M0002_set-up-tables`, etc. 
Inside each directory you'll find:
- the `migration.toml` file, which configures some of the options of this migration,
- (if SQL-based) the `up.sql` (and optionally `down.sql`) files, which is the SQL that will be executed for this migration (see `M0002_set-up-tables`), or
- (if Rust-based) the `mod.rs`, `up.rs`, and optionally `down.rs` files, which define more complex logic for applying or revering a migration (see `M0003_seed-permissions-and-roles` for an example).

> It is important to note that here, unlike in frameworks like Django, we must write the contents our migrations by hand (create a table, add fields, add indexes, ...). This ensures we have the highest control of our migration actions.


#### Appendix A.1 Creating a migration
First off, create an `.env` file in the root of the workspace. The following represents the correct username, password and database name for the development setup created with `./scripts/database/init-database.ps1`:

```bash
KOLOMONI_MIGRATIONS_DATABASE_URL_NORMAL_USER=postgres://kolomoni_migrator:kolomoni_migrator@localhost/stari_kolomoni

# Used when run_as_privileged_user is set in a migration.
# Usually done for the first-ever migration, see `M0001_prepare-database`.
KOLOMONI_MIGRATIONS_DATABASE_URL_PRIVILEGED_USER=postgres://postgres:postgres@localhost/stari_kolomoni
```

Then choose an appropriate name for your migration, e.g. `create_users_table`. You can then create a new migration by running:

```bash
cargo run --release --package kolomoni_migrations -- generate --migrations-directory "./kolomoni_migrations/migrations" --migration-name "foo-bar"

# Pass the --help flag for more information on how to not generate rollback scripts,
# how to create Rust migrations, etc.:
# 
#   cargo run --release --package kolomoni_migrations -- generate --help
```

This will create a new directory in the `kolomoni_migrations/migrations` directory, which will automatically be picked
up by the create and embedded into the binary at build-time. Now fill out `up.sql` and `down.sql` or equivalent Rust code 
with your migration. Keep in mind the order of the migrations matters: the new migration will be applied after all the previous migrations, of course.

To apply the migrations, see the migration status, etc., use the CLI:
```bash
cargo run --release --package kolomoni_migrations -- --help

cargo run --release --package kolomoni_migrations -- status

cargo run --release --package kolomoni_migrations -- up --help
cargo run --release --package kolomoni_migrations -- up --migrate-to-version 5

cargo run --release --package kolomoni_migrations -- down --help
cargo run --release --package kolomoni_migrations -- down --rollback-to-version 5
```


#### Appendix A.2 Adding new entities to `kolomoni_database`
After a migration adds some new tables that you want to interact with,
you need to add new entities into the `kolomoni_database` crate and write
the logic for querying and modifying them.

> Look at existing examples in the `kolomoni_database/src/entities` directory
> to get a feel for the structure.

Each entity is generally in its own directory inside the `kolomoni_database/src/entities`
directory, e.g. `word_meaning_english` for the "english word meaning" entity. Inside that
directory there should be at least four `.rs` files:
- `query.rs`, which contains all the logic for all possible queries of the
  given entity, represented as a single zero-sized struct with those queries as static functions on it.
  The struct name should be suffixed with `Query`, e.g. `EnglishWordMeaningQuery`.
  <br>Query functions on that struct should be async and always have `database_connection: &mut PgConnection` for their first parameter.
- `mutation.rs`, which is similar to `query.rs`, but contains all the logic for all possible creation, modification, and deletion operations of the given entity. Once again, all those operation should be present on a single zero-sized struct with functions, this time with the `Mutation` suffix, e.g. `EnglishWordMeaningMutation`.
  <br>Functions on that struct should, again, be async and always have `database_connection: &mut PgConnection` for their first parameter.
- `models.rs`, which has `pub(crate) mod internal` and `mod external` sub-modules in it.
  <br>Inside the private `internal` module you should define database models as they
  appear when interacting with the database itself. This means using base types
  such as `String`, `Uuid`, etc. Internal database types should be prefixed with
  `Internal` and suffixed with `Model`, e.g. `InternalEnglishWordMeaningModel`.
  <br>Inside the publicly re-exported `external` module you should define database
  models as they are passed into or returned from query and mutation functions (see above).
- `mod.rs`, which publicly re-exports (at least) the `models`, `mutation`, and `query` submodules.

> There are two more internal conventions here, which we won't go into:
> - the occasional `pub(crate) mod internal_weak` submodule, which is used for the
>   model types when fetching loosely-typed data (usually JSON) from the database, and
> - the occasional `pub(crate) mod internal_insert_only` submodule, which is used when the database
>   interaction model is different on insertion than on querying.
>
> Sometimes a directory called `queries` is also present. In it, you'll find `.sql`
> query files. We use these files for longer and more complex queries to avoid writing
> them all out inline in code.
>
> It's best to see the examples of this in action in `kolomoni_database/src/entities/word_meaning_english/models.rs`.
