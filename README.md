<h1 align="center">Stari Kolomoni Rust...end</h1>
<h6 align="center">Stari Kolomoni backend in Rust</h6>

This repository contains the backend API for the Stari Kolomoni open fantasy translation project.

---

## 1. Deployment
> *TODO :=)*

---


## 2. Development
This section describes how to set up for development. 
See `1. Deployment` for production instructions.


### 2.1 Software requirements
Before proceeding, verify that you have the following:
- `Rust 1.65` (or newer) - install with [*rustup*](https://rustup.rs/)
- `PostgreSQL 15.3` - install or extract portable to `PATH`.  
  > If on Windows, Powershell helper scripts are available in `scripts/database`. If on Windows you may also simply download a portable PostgreSQL installation and, instead of adding it to `PATH`, extract its `pgsql` directory to `scripts/database` (so that the `scripts/database/pgsql/bin` directory exists).
- `SeaORM CLI`, installed with `cargo install sea-orm-cli`. This will be used to manage database migrations.


### 2.2 Code style
> Important: *use nightly rustfmt*.

Use [nightly `rustfmt`](https://github.com/rust-lang/rustfmt) (`cargo +nighly fmt` or 
[equivalent IDE support](https://github.com/rust-lang/rustfmt#running-rustfmt-from-your-editor)) for formatting your
Rust code in this repository. Current rules can be seen in `rustfmt.toml` - you're welcome to adapt the rules to better
fit the codebase as the repository evolves.


### 2.3 Preparation
- Initialize the PostgreSQL database:  
  - on *Windows*: you may use the `scripts/database/init-database.ps1` and `scripts/database/run-database.ps1` scripts for easy setup and running. *This should only be used for development as it uses `--auth=trust`.* The generated user is `kolomon` (password `kolomon`) and the database name is `kolomondb`. The database will run on `127.0.0.1` on default port `5432`.
- Copy `data/configuration.TEMPLATE.toml` to `data/configuration.toml` and fill out the configuration fields (see comments in the template for explanations).
- Build the project by running `cargo build`. This will be a fresh build, so it might take a few minutes (around 2 minutes on a hard drive with a 16-core CPU at 3.6 GHz). Subsequent builds will be much faster.


### 2.4 Starting the backend server
To start the backend server, run `cargo run` (or `./target/debug/stari-kolomoni-backend`).


### 2.5 About database migrations
Migrations are managed by SeaORM and applied onto the database on startup (if any are un-applied).

The process of modifying the database schema or creating new tables is described below. 
SeaORM calls this order "schema-first", but that's just a recommendation - you *may* start 
by defining an entity first if you like.

> It is important to note that here, unlike in frameworks like Django, we must write our migrations by hand (create a table, add fields, add indexes, ...). This is all written in Rust code to be as database-agnostic as possible. See existing migrations as examples and additional guidance in `2.5.1 Create a migration`.


#### 2.5.1 Create a migration
Create a new migration with the appropriate name, e.g. `create_users_table` by running:

```bash
sea-orm-cli migrate generate --universal-time --migration-dir migrations create_users_table
```

This will create a new file in the `migrations/src` directory. Remove the `todo!()` calls and redundant comments and
write your own `up` and `down` implementation for your new migration. Keep in mind that the new migration will be applied last, after all existing migrations (see `migrations/src/lib.rs` for the order, but don't modify it unless you know what you're doing). 

As for a guide on writing migrations: see existing ones for some examples, but you can learn more about SeaORM migrations in 
[SeaORM - Writing Migration](https://www.sea-ql.org/SeaORM/docs/migration/writing-migration/).

The API documentation for the `SchemaManager` you're provided in the `up` and `down` methods 
is available on [docs.rs - sea_orm_migration::manager::SchemaManager](https://docs.rs/sea-orm-migration/0.11.3/sea_orm_migration/manager/struct.SchemaManager.html).

> Un-applied migrations will be performed when the backend starts, but you may perform the migration manually if you like by running:
> ```bash
> sea-orm-cli migrate up --database-url=postgres://username:password@host:port/database_name --migration-dir migrations
> ```
> 
> You can also check which migrations have already been applied by running:
> ```bash
> sea-orm-cli migrate status --database-url=postgres://username:password@host:port/database_name --migration-dir migrations
> ```


#### 2.5.2 Define the entity
Define the entity that matches your newly-created migration by creating a new file in `src/database/entities`. Look at existing entities for examples.

> **It is important that each entity has its own file and that names like `Model`, `Relation` and `ActiveModel` stay as is** - this 
> is a SeaORM implementation detail and the result of derive macros that allow us to write much less code, 
> see [SeaORM - Entity Structure](https://www.sea-ql.org/SeaORM/docs/generate-entity/entity-structure/) and, 
> if you're curious for what that expands to, [SeaORM - Expanded Entity Structure](https://www.sea-ql.org/SeaORM/docs/generate-entity/expanded-entity-structure/).


#### 2.5.3 Define relevant `Query` and `Mutation` methods
This is not enforced by SeaORM, but **actix routes and most other things aren't supposed to touch the defined entities in `database::entities` directly**.

Instead, here is a pattern for accessing and modifying anything in the database (names are examples):
- `database::query::user_permissions::UserPermissionsQuery` (or similar) is the struct that has public async methods that allow the rest of the application to query (find by id, name, ...) data, but not modify it. A single query struct should generally operate only on one entity.
- `database::mutation::users::UsersMutation` (or similar) is the struct that has public async methods the allow the rest of the application to modify the data of the given entity (or related).
- Query and mutation structs are then re-exported in `datbase/mutation/mod.rs` and `database/query/mod.rs` to eliminate unnecesarry nesting.

Familiarize yourself with examples on existing mutation and query structs. As a general rule, it probably makes sense to write query and mutation methods as we grow the application to need them, and *not* every operation up front after defining the entity.

As for the SeaORM documentation related to fetching and updating database data, the [SeaORM - Basic CRUD](https://www.sea-ql.org/SeaORM/docs/basic-crud/basic-schema/) chapter might be of much help.


### 2.6 `actix-web` and `EndpointResult`/`APIError` examples
We've introduced a few new types to easily `?`-return common `Result` errors,
see documentation for `api::v1::errors::APIError` for more information.
