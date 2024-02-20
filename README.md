<h1 align="center">Stari Kolomoni Backend</h1>

[![Test Status](https://img.shields.io/github/actions/workflow/status/Stari-kolomoni/kolomoni-backend/test.yml?branch=master&style=flat-square&logo=github&logoColor=white&label=doc%2C%20unit%20%26%20end-to-end%20tests)](https://github.com/Stari-kolomoni/kolomoni-backend/actions/workflows/test.yml)
[![Licensed under GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blue?style=flat-square)](https://github.com/Stari-kolomoni/kolomoni-backend/blob/master/LICENSE.md)
[![Development Docs](https://img.shields.io/badge/development_docs-here-orange)](https://stari-kolomoni.github.io/kolomoni-backend/)


This repository contains the full backend for the Stari Kolomoni open fantasy translation project.

<br>
<br>

# 1. Deployment
> *TODO :=)*



# 2. Development
This section describes how to set up for development. 
See `1. Deployment` for production instructions.



## 2.1 Software requirements
Before proceeding, verify that you have the following:
- `Rust 1.65` or newer - install with [*rustup*](https://rustup.rs/)
- `PostgreSQL 15.3` or newer - install or extract portable to `PATH`.  
  > If on Windows, Powershell helper scripts are available in `scripts/database`. To simplify things you can also simply download a portable PostgreSQL archive instead of installing it. You can then, instead of adding it to `PATH`, extract its `pgsql` directory to `scripts/database` (i.e. so that the `scripts/database/pgsql/bin` directory exists).
- `SeaORM CLI`, installed with `cargo install sea-orm-cli`. This will be used to manage database migrations.



## 2.2 Code style and IDE setup
> Important: *use nightly rustfmt*.

Use [nightly `rustfmt`](https://github.com/rust-lang/rustfmt) (`cargo +nighly fmt` or 
[equivalent IDE support](https://github.com/rust-lang/rustfmt#running-rustfmt-from-your-editor)) for formatting your
Rust code in this repository. Current rules can be seen in `rustfmt.toml` - you're welcome to adapt the rules to better
fit the codebase as the repository evolves.


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

Alongside `rust-analyzer` and this configuration, we would like to suggest the following extensions:
- **(highly recommended)** [EditorConfig](https://marketplace.visualstudio.com/items?itemName=EditorConfig.EditorConfig),
- *(good-to-have)* [Even Better TOML](https://marketplace.visualstudio.com/items?itemName=tamasfe.even-better-toml), and
- *(optional; highlights comments)* [Better Comments](https://marketplace.visualstudio.com/items?itemName=aaron-bond.better-comments).

For Better Comments, the following configuration might be of use — add this to `.vscode/settings.json` after installing the extension:

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



## 2.3 Setting up the database and `configuration.toml`

First off, initialize the PostgreSQL database.
If you're on *Windows*, you can use the `scripts/database/init-database.ps1` and `scripts/database/run-database.ps1` scripts for easy setup and running.
*Note that this script should only be used for development as it uses `--auth=trust`.*
The generated user is `kolomon` (password `kolomon`) and the database name is `kolomondb`. 
The database will run on `127.0.0.1` on default port `5432`.

Then, copy `data/configuration.TEMPLATE.toml` to `data/configuration.toml` and 
fill out the configuration fields (see comments in the template for explanations).

Build the project by running `cargo build`. 
This will be a fresh build, so it might take a few minutes (around 2 minutes on a hard drive with a 16-core CPU at 3.6 GHz).
Subsequent builds will be much faster.


## 2.4 Starting the backend server
To start the backend server, execute `cargo run` (or run the binary in `./target/debug`).


## Appendix

### Appendix A. About database migrations
Migrations are managed by [SeaORM](https://www.sea-ql.org/SeaORM/) and are applied onto the database on startup 
(if needed).

The process of modifying the database schema or creating new tables is described below. 
SeaORM calls this order "schema-first", and we tend to follow this recommendation.

> It is important to note that here, unlike in frameworks like Django, we must write our migrations by hand (create a table, add fields, add indexes, ...). This is all written in Rust code to be as database-agnostic as possible. See existing migrations as examples and additional guidance in `A.1 Create a migration`.


#### Appendix A.1 Creating a migration
First off, create an `.env` file in the root of the cloned repository and set the `DATABASE_URL` environment variable.
The following represents the correct username, password and database name for the development setup created with `./scripts/database/init-database.ps1`:

```bash
DATABASE_URL=postgres://kolomon:kolomon@localhost/kolomondb
```

Then choose an appropriate name for your migration, e.g. `create_users_table`. You can then create a new migration by running:

```bash
sea-orm-cli migrate generate --universal-time --migration-dir ./kolomoni_migrations create_users_table
```

This will create a new file in the `./kolomoni_migrations/src` directory. 
Remove the `todo!()` calls and redundant comments. 
Then write your own `up` and `down` implementation for your new migration. 
Keep in mind that the newly-added migration will be applied *last*, after all existing migrations 
(see `migrations/src/lib.rs` for the order - don't modify it unless you know what you're doing). 

As for a guide on writing migrations: see existing ones for some examples, but you can learn more about SeaORM migrations in 
[SeaORM - Writing Migration](https://www.sea-ql.org/SeaORM/docs/migration/writing-migration/).

The API documentation for the `SchemaManager` parameter you're provided in the `up` and `down` methods 
is available on [docs.rs - sea_orm_migration::manager::SchemaManager](https://docs.rs/sea-orm-migration/latest/sea_orm_migration/manager/struct.SchemaManager.html).

> Un-applied migrations will be performed when the backend starts, but you may perform the migration manually if you like by running:
> ```bash
> sea-orm-cli migrate up --database-url=postgres://username:password@host:port/database_name --migration-dir migrations
> ```
> 
> You can also check which migrations have already been applied by running:
> ```bash
> sea-orm-cli migrate status --database-url=postgres://username:password@host:port/database_name --migration-dir migrations
> ```


#### Appendix A.2 Generating entity code from the modified schema
> The following takes place in the `kolomoni_database` crate.

The `entities` module is supposed to be auto-generated by SeaORM *and as such you should not modify it*.
If you added a migration and wish to update the entities to the current schema, you must first apply the migrations (see above). Afterwards, run:

```bash
sea-orm-cli generate entity --output-dir=./kolomoni_database/src/entities --expanded-format
```

> *Again: do not modify this auto-generated code! Your changes will be overwritten the next time someone runs this command.*


#### Appendix A.3 Defining relevant queries and mutations on top of the generated entities
> The following takes place in the `kolomoni_database` crate.

**While not enforced by SeaORM, we want to avoid non-database code (especially actix routes) touching the defined entities in `database::entities` directly**.

Instead of querying and updating those entities manually, here is a pattern for accessing and modifying anything in the database (names are examples, adapt to the relevant entity):
- `crate::query::user_permission::UserPermissionQuery` is the struct that has public async methods that 
  allow the rest of the application to query data (find by id, name, ...), *but not to modify it*. 
  A single query struct should generally operate only on one entity.
- `crate::mutation::user_permission::UserPermissionMutation` is the struct that has public async methods that
  allow the rest of the application to *modify the data of the given entity* (or related entities, up to you).

Query and mutation structs are then re-exported in `src/mutation.rs` and `src/query.rs` to eliminate unnecesarry nesting.
As a general rule, it probably makes sense to write query and mutation methods as we grow the application to need them, and *not* every operation up front after defining the entity.

As for the SeaORM documentation related to fetching and updating database data, the [SeaORM - Basic CRUD](https://www.sea-ql.org/SeaORM/docs/basic-crud/basic-schema/) chapter might be of much help.



### Appendix B. `actix-web` and `EndpointResult`/`APIError` examples
We've introduced a few new types to easily `?`-return common `Result` errors,
see documentation for `api::v1::errors::APIError` for more information.
