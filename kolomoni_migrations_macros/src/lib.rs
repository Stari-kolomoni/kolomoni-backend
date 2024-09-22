use std::path::{Path, PathBuf};

use kolomoni_migrations_core::sha256::Sha256Hash;
use migrations::{scan_for_migrations, ScannedMigrationScript};
use proc_macro::TokenStream;
use proc_macro_error2::{abort_call_site, proc_macro_error};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::Parse,
    parse_macro_input,
    parse_quote,
    ItemFn,
    LitStr,
    ReturnType,
    Signature,
    Token,
    Type,
    Visibility,
};

pub(crate) mod migrations;


struct EmbedMigrationsArgs {
    migrations_directory_relative_to_caller_source_file: PathBuf,

    migrations_directory_absolute: PathBuf,
}

impl Parse for EmbedMigrationsArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let migrations_directory_path_relative_to_caller_crate_root_lit = input.parse::<LitStr>()?;

        input.parse::<Token![,]>()?;

        let caller_source_file_relative_path_to_crate_root_lit = input.parse::<LitStr>()?;

        input.parse::<Token![,]>()?;

        let relative_path_from_macro_crate_to_migrations_crate = input.parse::<LitStr>()?;


        let caller_relative_path_to_crate_root =
            PathBuf::from(caller_source_file_relative_path_to_crate_root_lit.value());

        let proc_crate_relative_path_to_caller_root =
            PathBuf::from(relative_path_from_macro_crate_to_migrations_crate.value());


        let migrations_directory_relative_to_caller_source_file = caller_relative_path_to_crate_root
            .join(migrations_directory_path_relative_to_caller_crate_root_lit.value());

        let migrations_directory_absolute = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(proc_crate_relative_path_to_caller_root)
            .join(migrations_directory_path_relative_to_caller_crate_root_lit.value());

        Ok(Self {
            migrations_directory_relative_to_caller_source_file,
            migrations_directory_absolute,
        })
    }
}


fn sha256_to_u8_array_token_stream(sha: &Sha256Hash) -> proc_macro2::TokenStream {
    let mut individual_byte_tokens = Vec::with_capacity(32);

    for byte in sha.as_slice() {
        individual_byte_tokens.push(quote! { #byte })
    }

    assert_eq!(individual_byte_tokens.len(), 32);

    quote! {
        [#(#individual_byte_tokens),*]
    }
}


/// Embeds a set of SQL and/or Rust migration scripts into the binary at compile-time.
///
/// This procedural macro expects three parameters, all of them string literals, in the following order:
/// - name of the migrations directory (relative to the root of the caller crate),
/// - relative path from the current source file to the root of the caller crate, and
/// - relative path from the `kolomoni_migrations_macros` crate root to the caller crate root.
///
/// For example, when using this in `kolomoni_migrations`, which is adjacent to `kolomoni_migrations_macros`,
/// we use the following:
/// ```no_run
/// use kolomoni_migrations_macros::embed_migrations;
///
/// embed_migrations!(
///     // This is the name of the directory that contains the actual SQL / Rust migrations.
///     "migrations",
///     // This is a relative path from the caller file to the root of the crate. In our use case
///     // we call this in `kolomoni_migrations/src/main.rs`, so we need to ascend once.
///     "..",
///     // This is a path from the `kolomoni_migrations_macros` crate to `kolomoni_migrations`,
///     // which is the caller crate. As they sit adjacently in a workspace, this is relatively simple.
///     "../kolomoni_migrations"
/// );
/// ```
///
/// When <https://github.com/rust-lang/rust/issues/54725> is stabilized, we will be able to simplify this greatly.
#[proc_macro]
#[proc_macro_error]
pub fn embed_migrations(input: TokenStream) -> TokenStream {
    let macro_args = parse_macro_input!(input as EmbedMigrationsArgs);


    let local_migrations = match scan_for_migrations(&macro_args.migrations_directory_absolute) {
        Ok(migrations) => migrations,
        Err(error) => {
            abort_call_site!(
                "failed to load migrations from directory {}: {}",
                macro_args
                    .migrations_directory_relative_to_caller_source_file
                    .display(),
                error
            );
        }
    };


    let mut code_to_prepend_to_module = Vec::new();
    let mut embedded_migration_constructions = Vec::new();


    for migration in local_migrations {
        let migration_directory_name = migration.identifier().to_directory_name();
        let migration_directory_path_relative_to_caller_source_file = macro_args
            .migrations_directory_relative_to_caller_source_file
            .join(migration_directory_name);


        let configuration = migration.configuration();


        let mut rust_module_import = None;


        let up_concrete_script = match migration.up() {
            ScannedMigrationScript::Sql(sql_up) => {
                let sql_script_string = sql_up.sql.as_str();
                let sql_sha256_hash_token_stream =
                    sha256_to_u8_array_token_stream(&sql_up.sha256_hash);


                let sql_script_path =
                    migration_directory_path_relative_to_caller_source_file.join("up.sql");
                let sql_script_path_str = sql_script_path.to_str().unwrap_or_else(|| {
                    abort_call_site!("failed to parse migration due to non-UTF-8 path")
                });

                code_to_prepend_to_module.push(quote! {
                    const _: &str = include_str!(#sql_script_path_str);
                });
                let sql_sha256_hash_construction = quote! {
                    Sha256Hash::from_bytes(#sql_sha256_hash_token_stream)
                };

                quote! {
                    EmbeddedMigrationScript::new_sql(
                        #sql_script_string,
                        #sql_sha256_hash_construction
                    )
                }
            }
            ScannedMigrationScript::Rust(rust_up) => {
                let sha_hash_bytes_token_stream =
                    sha256_to_u8_array_token_stream(&rust_up.sha256_hash);


                let injected_module_path_relative_to_caller =
                    migration_directory_path_relative_to_caller_source_file.join("mod.rs");

                let injected_module_path_relative_to_caller_str =
                    injected_module_path_relative_to_caller
                        .to_str()
                        .unwrap_or_else(|| {
                            abort_call_site!("failed to parse migration due to non-UTF-8 path")
                        });

                let injected_module_name_ident = format_ident!(
                    "_migration_{:04}",
                    migration.identifier().version as u64
                );

                rust_module_import = Some(quote! {
                    #[path = #injected_module_path_relative_to_caller_str]
                    mod #injected_module_name_ident;
                });

                let migration_function_path: syn::Path = parse_quote! {
                    #injected_module_name_ident::up
                };
                let migration_file_sha256_hash = quote! {
                    Sha256Hash::from_bytes(#sha_hash_bytes_token_stream)
                };

                quote! {
                    EmbeddedMigrationScript::new_rust(
                        #migration_function_path,
                        #migration_file_sha256_hash
                    )
                }
            }
        };

        let down_concrete_script = if let Some(down) = migration.down() {
            match down {
                ScannedMigrationScript::Sql(sql_down) => {
                    let sql_script_string = sql_down.sql.as_str();
                    let sql_sha256_hash_token_stream =
                        sha256_to_u8_array_token_stream(&sql_down.sha256_hash);


                    let sql_script_path =
                        migration_directory_path_relative_to_caller_source_file.join("down.sql");
                    let sql_script_path_str = sql_script_path.to_str().unwrap_or_else(|| {
                        abort_call_site!("failed to parse migration due to non-UTF-8 path")
                    });

                    code_to_prepend_to_module.push(quote! {
                        const _: &str = include_str!(#sql_script_path_str);
                    });
                    let sql_sha256_hash_construction = quote! {
                        Sha256Hash::from_bytes(#sql_sha256_hash_token_stream)
                    };

                    quote! {
                        Some(EmbeddedMigrationScript::new_sql(
                            #sql_script_string,
                            #sql_sha256_hash_construction
                        ))
                    }
                }
                ScannedMigrationScript::Rust(rust_down) => {
                    let sha_hash_bytes_token_stream =
                        sha256_to_u8_array_token_stream(&rust_down.sha256_hash);


                    let injected_module_path_relative_to_caller =
                        migration_directory_path_relative_to_caller_source_file.join("mod.rs");

                    let injected_module_path_relative_to_caller_str =
                        injected_module_path_relative_to_caller
                            .to_str()
                            .unwrap_or_else(|| {
                                abort_call_site!("failed to parse migration due to non-UTF-8 path")
                            });

                    let injected_module_name_ident = format_ident!(
                        "_migration_{:04}",
                        migration.identifier().version as u64
                    );


                    rust_module_import = Some(quote! {
                        #[path = #injected_module_path_relative_to_caller_str]
                        mod #injected_module_name_ident;
                    });

                    let migration_function_path: syn::Path = parse_quote! {
                        #injected_module_name_ident::down
                    };
                    let migration_file_sha256_hash = quote! {
                        Sha256Hash::from_bytes(#sha_hash_bytes_token_stream)
                    };

                    quote! {
                        Some(EmbeddedMigrationScript::new_rust(
                            #migration_function_path,
                            #migration_file_sha256_hash
                        ))
                    }
                }
            }
        } else {
            quote! { None }
        };


        if let Some(rust_module_import) = rust_module_import {
            code_to_prepend_to_module.push(rust_module_import);
        }


        let migration_version = migration.identifier().version;
        let migration_name = &migration.identifier().name;

        embedded_migration_constructions.push(quote! {
            EmbeddedMigration::new(
                MigrationIdentifier::new(#migration_version, #migration_name),
                #configuration,
                #up_concrete_script,
                #down_concrete_script
            )
        });
    }


    quote! {
        #(#code_to_prepend_to_module)*

        use kolomoni_migrations_core::migrations::MigrationManager;
        use kolomoni_migrations_core::migrations::EmbeddedMigration;
        use kolomoni_migrations_core::migrations::EmbeddedMigrationScript;
        use kolomoni_migrations_core::identifier::MigrationIdentifier;
        use kolomoni_migrations_core::sha256::Sha256Hash;

        pub fn manager() -> MigrationManager {
            MigrationManager::new_embedded(
                vec![
                    #(#embedded_migration_constructions),*
                ]
            )
        }
    }
    .into()
}


/// Validates and prepares the `async fn up` function of a given migration script.
///
/// It ensures the visibility is `pub(super)`, that the function is `async`,
/// that it is named `up`, etc. Internally a shim is created to bridge the gap between
/// a boxed future that is used in `kolomoni_migrations_core` and this "normal" async function.
///
/// The following is an example of using this in an `up.rs` file of a migration:
/// ```no_run
/// use kolomoni_migrations_core::errors::MigrationApplyError;
/// use sqlx::PgConnection;
///
/// #[kolomoni_migrations_macros::up]
/// pub(super) async fn up(database_connection: &mut PgConnection) -> Result<(), MigrationApplyError> {
///     // ... migration ...
///     Ok(())
/// }
///
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
pub fn up(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let Ok(up_function) = syn::parse::<ItemFn>(input) else {
        abort_call_site!(
            "the kolomoni_migrations_macros::up macro must be attached to an async function"
        );
    };


    // Enforce that the function is `pub`.
    if !matches!(&up_function.vis, Visibility::Public(_)) {
        abort_call_site!(
            "the annotated function must be marked as pub, not {:?}",
            up_function.vis
        );
    };


    // Enforce that the function is `async`.
    if up_function.sig.asyncness.is_none() {
        abort_call_site!("the annotated function must be an async function");
    }


    // Enforce that the function is named `up`.
    if up_function.sig.ident != "up" {
        abort_call_site!(
            "the function name must equal \"up\", not \"{}\"",
            up_function.sig.ident
        );
    }

    // Enforce that the function return type is `Result<(), MigrationApplyError>`.
    let ReturnType::Type(_, return_type) = &up_function.sig.output else {
        abort_call_site!("the function return type must be Result<(), MigrationApplyError>, not ()");
    };

    let Type::Path(return_type_path) = *return_type.clone() else {
        abort_call_site!(
            "the function return type must be Result<(), MigrationApplyError>, not {}",
            return_type.to_token_stream()
        );
    };

    // This is not foolproof, but it's an okay attempt.
    let Some(last_return_type_path_segment) = return_type_path.path.segments.last() else {
        abort_call_site!(
            "the function return type must be Result<(), MigrationApplyError>, not {}",
            return_type.to_token_stream()
        );
    };

    if last_return_type_path_segment.ident != "Result" {
        abort_call_site!(
            "the function return type must be Result<(), MigrationApplyError>, not {}",
            return_type.to_token_stream()
        );
    }



    let wrapped_inner_function_ident = format_ident!("_{}", up_function.sig.ident);
    let wrapped_inner_function = {
        let updated_signature = Signature {
            ident: wrapped_inner_function_ident.clone(),
            ..up_function.sig
        };

        ItemFn {
            vis: Visibility::Inherited,
            sig: updated_signature,
            ..up_function
        }
    };


    quote! {
        #wrapped_inner_function

        pub fn up<'c>(context: MigrationContext<'c>) ->
            std::pin::Pin<Box<
                dyn std::future::Future<
                    Output = Result<(), kolomoni_migrations_core::errors::MigrationApplyError>
                > + 'c
            >>
        {
            Box::pin(#wrapped_inner_function_ident(context))
        }
    }
    .into()
}



/// Validates and prepares the `async fn down` function of a given migration script.
///
/// It ensures the visibility is `pub(super)`, that the function is `async`,
/// that it is named `down`, etc. Internally a shim is created to bridge the gap between
/// a boxed future that is used in `kolomoni_migrations_core` and this "normal" async function.
///
/// The following is an example of using this in an `down.rs` file of a migration:
/// ```no_run
/// use kolomoni_migrations_core::errors::MigrationRollbackError;
/// use sqlx::PgConnection;
///
/// #[kolomoni_migrations_macros::down]
/// pub(super) async fn up(database_connection: &mut PgConnection) -> Result<(), MigrationRollbackError> {
///     // ... rollback ...
///     Ok(())
/// }
///
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
pub fn down(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let Ok(down_function) = syn::parse::<ItemFn>(input) else {
        abort_call_site!(
            "the kolomoni_migrations_macros::down macro must be attached to an async function"
        );
    };


    // Enforce that the function is `pub`.
    if !matches!(&down_function.vis, Visibility::Public(_)) {
        abort_call_site!(
            "the annotated function must be marked as pub, not {:?}",
            down_function.vis
        );
    };


    // Enforce that the function is `async`.
    if down_function.sig.asyncness.is_none() {
        abort_call_site!("the annotated function must be an async function");
    }


    // Enforce that the function is named `down`.
    if down_function.sig.ident != "down" {
        abort_call_site!(
            "the function name must equal \"down\", not \"{}\"",
            down_function.sig.ident
        );
    }

    // Enforce that the function return type is `Result<(), MigrationApplyError>`.
    let ReturnType::Type(_, return_type) = &down_function.sig.output else {
        abort_call_site!(
            "the function return type must be Result<(), MigrationRollbackError>, not ()"
        );
    };

    let Type::Path(return_type_path) = *return_type.clone() else {
        abort_call_site!(
            "the function return type must be Result<(), MigrationRollbackError>, not {}",
            return_type.to_token_stream()
        );
    };

    // This is not foolproof, but it's an okay attempt.
    let Some(last_return_type_path_segment) = return_type_path.path.segments.last() else {
        abort_call_site!(
            "the function return type must be Result<(), MigrationRollbackError>, not {}",
            return_type.to_token_stream()
        );
    };

    if last_return_type_path_segment.ident != "Result" {
        abort_call_site!(
            "the function return type must be Result<(), MigrationRollbackError>, not {}",
            return_type.to_token_stream()
        );
    }



    let wrapped_inner_function_ident = format_ident!("_{}", down_function.sig.ident);
    let wrapped_inner_function = {
        let updated_signature = Signature {
            ident: wrapped_inner_function_ident.clone(),
            ..down_function.sig
        };

        ItemFn {
            vis: Visibility::Inherited,
            sig: updated_signature,
            ..down_function
        }
    };


    quote! {
        #wrapped_inner_function

        pub fn down<'c>(context: MigrationContext<'c>) ->
            std::pin::Pin<Box<
                dyn std::future::Future<
                    Output = Result<(), kolomoni_migrations_core::errors::MigrationRollbackError>
                > + 'c
            >>
        {
            Box::pin(#wrapped_inner_function_ident(context))
        }
    }
    .into()
}
