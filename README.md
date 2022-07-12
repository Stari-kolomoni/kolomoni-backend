# Stari Kolomoni Rustend

Kolomoni Backend ... but in Rust. Because that's a good idea.


## Reqirements
Uses
 - Cargo 1.62.0
 - Actix
 - Diesel

Also requires external Diesel CLI. Install it with

```shell
cargo install diesel_cli --no-default-features --features postgres
```

## Database

Database is PostgreSQL server, maintained trough Diesel.

Migrations are run with

```shell
diesel migration run
```