use std::{collections::HashMap, rc::Rc};

use analyzer::{models::InternalCategoryId, SeedDataset};
use clap::Parser;
use cli::{CliArguments, CliCommand, SeedFromSpreadsheetCommandArguments};
use kolomoni_api_client::{
    api::dictionary::categories::{CategoryFieldsToUpdate, CategoryToCreate},
    authentication::AccessToken,
    ApiServer,
    ApiServerOptions,
    AuthenticatedClient,
    ServerHost,
};
use kolomoni_core::ids::CategoryId;
use miette::{Context, IntoDiagnostic};
use parser::SeedSpreadsheetsParser;

use crate::analyzer::insert_only_set::InternalId;

mod analyzer;
mod cli;
mod parser;


fn build_api_client(
    server_host_or_ip: &str,
    server_port: usize,
    access_token: &str,
) -> miette::Result<AuthenticatedClient> {
    let api_server = Rc::new(ApiServer::new(
        ServerHost::DomainName(format!("{}:{}", server_host_or_ip, server_port)),
        ApiServerOptions { use_https: false },
    ));

    let client = kolomoni_api_client::Client::new(&api_server)
        .into_diagnostic()
        .wrap_err("failed to initialize API client")?;


    let api_authentication = Rc::new(AccessToken::new(access_token.to_owned()));

    Ok(client.with_authentication(&api_authentication))
}


async fn main_async(seed_arguments: SeedFromSpreadsheetCommandArguments) -> miette::Result<()> {
    let authenticated_api_client = build_api_client(
        &seed_arguments.server_host_or_ip,
        seed_arguments.server_port,
        &seed_arguments.access_token,
    )?;


    let spreadsheets_parser = SeedSpreadsheetsParser::from_csv_file_paths(
        &seed_arguments.input_file_path_with_translations,
        &seed_arguments.input_file_path_with_categories,
    )
    .into_diagnostic()
    .wrap_err("failed to initialize spreadsheets parser")?;


    let parsed_data = SeedDataset::parse(spreadsheets_parser)
        .into_diagnostic()
        .wrap_err("failed to parse spreadsheet")?;



    // Insert all categories and note down their public UUIDs.
    let mut internal_to_public_category_id: HashMap<InternalCategoryId, CategoryId> = HashMap::new();

    let categories_api_client = authenticated_api_client.categories();
    for parsed_category in parsed_data.categories().read_inner().values() {
        let new_category = categories_api_client
            .create_category(CategoryToCreate {
                english_category_name: parsed_category.english_name.clone(),
                slovene_category_name: parsed_category.slovene_name.clone(),
                // Will be set in another pass.
                parent_category_id: None,
            })
            .await
            .into_diagnostic()
            .wrap_err("failed to create category on the server")?;

        internal_to_public_category_id.insert(parsed_category.internal_id(), new_category.id);
    }

    for parsed_category in parsed_data.categories().read_inner().values() {
        let Some(parent_category_internal_id) = parsed_category.parent_category else {
            continue;
        };


        let associated_real_category_id = internal_to_public_category_id
            .get(&parsed_category.internal_id())
            .expect(
                "expected all parsed categories to have been inserted already (can't find self)",
            );

        let associated_real_parent_category_id = internal_to_public_category_id
            .get(&parent_category_internal_id)
            .expect(
                "expected all parsed categories to have been inserted already (can't find parent)",
            );


        categories_api_client
            .update_category(
                *associated_real_category_id,
                CategoryFieldsToUpdate {
                    new_english_name: None,
                    new_slovene_name: None,
                    new_parent_category_id: Some(Some(*associated_real_parent_category_id)),
                },
            )
            .await
            .into_diagnostic()
            .wrap_err("failed to set parent category on the server")?;
    }


    // TODO continue from here
    // Insert all slovene words and their meanings and note down their public UUIDs.
    let mut internal_to_public_slovene_word_id = HashMap::new();



    todo!();

    // Insert all english words and their meanings and note down their public UUIDs.
    todo!();

    // Establish translation relationships between english and slovene word meanings.
    todo!();
}

fn main() -> miette::Result<()> {
    let cli_args = CliArguments::parse();


    match cli_args.command {
        CliCommand::SeedFromSpreadsheetCommand(seed_arguments) => {
            let async_runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .into_diagnostic()
                .wrap_err("failed to build tokio multi-threaded async runtime")?;

            async_runtime.block_on(main_async(seed_arguments))
        }
    }
}
