use std::{collections::HashMap, rc::Rc};

use analyzer::{
    insert_only_set::InternalId,
    models::{
        InternalCategoryId,
        InternalEnglishWordId,
        InternalEnglishWordMeaningId,
        InternalSloveneWordId,
        InternalSloveneWordMeaningId,
    },
    SeedDataset,
};
use clap::Parser;
use cli::{CliArguments, CliCommand, SeedFromSpreadsheetCommandArguments};
use kolomoni_api_client::{
    api::dictionary::{
        categories::{CategoryFieldsToUpdate, CategoryToCreate},
        english::{EnglishWordMeaningToCreate, EnglishWordToCreate},
        slovene::{SloveneWordMeaningToCreate, SloveneWordToCreate},
        translation::TranslationRelationshipToCreate,
    },
    authentication::AccessToken,
    ApiServer,
    ApiServerOptions,
    AuthenticatedClient,
    ServerHost,
};
use kolomoni_core::ids::{
    CategoryId,
    EnglishWordId,
    EnglishWordMeaningId,
    SloveneWordId,
    SloveneWordMeaningId,
};
use miette::{miette, Context, IntoDiagnostic};
use parser::SeedSpreadsheetsParser;


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

    for parsed_category in parsed_data.categories().values() {
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

    for parsed_category in parsed_data.categories().values() {
        let Some(parent_category) = parsed_category
            .parent_category(parsed_data.categories())
            .into_diagnostic()
            .wrap_err("failed to obtain parent category")?
        else {
            continue;
        };


        let associated_real_category_id = internal_to_public_category_id
            .get(&parsed_category.internal_id())
            .expect(
                "expected all parsed categories to have been inserted already (can't find self)",
            );

        let associated_real_parent_category_id = internal_to_public_category_id
            .get(&parent_category.internal_id())
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


    // Insert all slovene words and their meanings and note down their public UUIDs.
    let mut internal_to_public_slovene_word_id: HashMap<InternalSloveneWordId, SloveneWordId> =
        HashMap::new();

    let mut internal_to_public_slovene_word_meaning_id: HashMap<
        InternalSloveneWordMeaningId,
        SloveneWordMeaningId,
    > = HashMap::new();


    let slovene_api_client = authenticated_api_client.slovene_dictionary();

    for slovene_word in parsed_data.slovene_words().values() {
        let new_slovene_word = slovene_api_client
            .create_slovene_word(SloveneWordToCreate {
                lemma: slovene_word.lemma.clone(),
            })
            .await
            .into_diagnostic()
            .wrap_err("failed to create new slovene word")?;


        internal_to_public_slovene_word_id.insert(slovene_word.internal_id(), new_slovene_word.id);
    }

    for slovene_word_meaning in parsed_data.slovene_word_meanings().values() {
        let associated_slovene_word = internal_to_public_slovene_word_id
            .get(&slovene_word_meaning.word_internal_id())
            .ok_or_else(|| {
                miette!("expected to have already created the associated slovene word")
            })?;

        let new_slovene_word_meaning = slovene_api_client
            .create_slovene_word_meaning(
                *associated_slovene_word,
                SloveneWordMeaningToCreate {
                    abbreviation: slovene_word_meaning.abbreviation.clone(),
                    disambiguation: slovene_word_meaning.disambiguation.clone(),
                    description: slovene_word_meaning.description.clone(),
                },
            )
            .await
            .into_diagnostic()
            .wrap_err("failed to create new slovene word meaning")?;

        internal_to_public_slovene_word_meaning_id.insert(
            slovene_word_meaning.internal_id(),
            new_slovene_word_meaning.meaning_id,
        );


        let connected_categories = slovene_word_meaning
            .categories(parsed_data.categories())
            .into_diagnostic()
            .wrap_err("expected to have already created all associated categories")?;

        for category in connected_categories {
            let target_category_id = internal_to_public_category_id
                .get(&category.internal_id())
                .ok_or_else(|| {
                    miette!("expected to have already created all associated categories")
                })?;

            slovene_api_client
                .link_category_to_slovene_word_meaning(
                    *associated_slovene_word,
                    new_slovene_word_meaning.meaning_id,
                    *target_category_id,
                )
                .await
                .into_diagnostic()
                .wrap_err("failed to link slovene word meaning with a category")?;
        }
    }


    // Insert all english words and their meanings and note down their public UUIDs.
    let mut internal_to_public_english_word_id: HashMap<InternalEnglishWordId, EnglishWordId> =
        HashMap::new();

    let mut internal_to_public_english_word_meaning_id: HashMap<
        InternalEnglishWordMeaningId,
        EnglishWordMeaningId,
    > = HashMap::new();

    let english_api_client = authenticated_api_client.english_dictionary();

    for english_word in parsed_data.english_words().values() {
        let new_english_word = english_api_client
            .create_english_word(EnglishWordToCreate {
                lemma: english_word.lemma.clone(),
            })
            .await
            .into_diagnostic()
            .wrap_err("failed to create new english word")?;


        internal_to_public_english_word_id.insert(english_word.internal_id(), new_english_word.id);
    }

    for english_word_meaning in parsed_data.english_word_meanings().values() {
        let associated_english_word = internal_to_public_english_word_id
            .get(&english_word_meaning.word_internal_id())
            .ok_or_else(|| {
                miette!("expected to have already created the associated english word")
            })?;

        let new_english_word_meaning = english_api_client
            .create_english_word_meaning(
                *associated_english_word,
                EnglishWordMeaningToCreate {
                    abbreviation: english_word_meaning.abbreviation.clone(),
                    description: english_word_meaning.description.clone(),
                    disambiguation: english_word_meaning.disambiguation.clone(),
                },
            )
            .await
            .into_diagnostic()
            .wrap_err("failed to create new english word meaning")?;


        internal_to_public_english_word_meaning_id.insert(
            english_word_meaning.internal_id(),
            new_english_word_meaning.meaning_id,
        );


        let connected_categories = english_word_meaning
            .categories(parsed_data.categories())
            .into_diagnostic()
            .wrap_err("expected to have already created all associated categories")?;

        for category in connected_categories {
            let target_category_id = internal_to_public_category_id
                .get(&category.internal_id())
                .ok_or_else(|| {
                    miette!("expected to have already created all associated categories")
                })?;

            english_api_client
                .link_category_to_english_word_meaning(
                    *associated_english_word,
                    new_english_word_meaning.meaning_id,
                    *target_category_id,
                )
                .await
                .into_diagnostic()
                .wrap_err("failed to link english word meaning with a category")?;
        }
    }


    // Establish translation relationships between english and slovene word meanings.

    let translations_api_client = authenticated_api_client.translations();

    for translation in parsed_data.translations().values() {
        let associated_slovene_word_meaning_id = internal_to_public_slovene_word_meaning_id
            .get(&translation.slovene_word_meaning)
            .ok_or_else(|| {
                miette!("expected to have already created the associated slovene word meaning")
            })?;

        let associated_english_word_meaning_id = internal_to_public_english_word_meaning_id
            .get(&translation.english_word_meaning)
            .ok_or_else(|| {
                miette!("expected to have already created the associated english word meaning")
            })?;


        translations_api_client
            .create_translation_relationship(TranslationRelationshipToCreate {
                slovene_word_meaning: *associated_slovene_word_meaning_id,
                english_word_meaning: *associated_english_word_meaning_id,
            })
            .await
            .into_diagnostic()
            .wrap_err("failed to create translation relationship")?;
    }


    Ok(())
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
