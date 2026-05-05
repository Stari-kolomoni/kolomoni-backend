use std::{collections::HashMap, sync::Arc};

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
        categories::{
            CategoryFieldsToUpdate,
            CategoryToCreate,
            DictionaryCategoriesAuthenticatedEndpoints,
        },
        english::{
            EnglishDictionaryAuthenticatedEndpoints,
            EnglishWordMeaningToCreate,
            EnglishWordToCreate,
        },
        slovene::{
            SloveneDictionaryAuthenticatedEndpoints,
            SloveneWordMeaningToCreate,
            SloveneWordToCreate,
        },
        translation::{TranslationAuthenticatedEndpoints, TranslationRelationshipToCreate},
    },
    authentication::ClientAuthentication,
    client::{AuthenticatedKolomoniClient, UnauthenticatedKolomoniClient},
    server::{ApiServerOptions, KolomoniApiServer, ServerHost},
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
) -> miette::Result<AuthenticatedKolomoniClient> {
    let api_server = Arc::new(
        KolomoniApiServer::new_from_server_url(
            ServerHost::DomainName(format!("{}:{}", server_host_or_ip, server_port)),
            ApiServerOptions { use_https: false },
        )
        .into_diagnostic()
        .wrap_err("failed to construct API server URL")?,
    );

    let client = UnauthenticatedKolomoniClient::new(api_server)
        .into_diagnostic()
        .wrap_err("failed to initialize API client")?;


    let api_authentication = ClientAuthentication::new(access_token.to_owned(), "DUMMY".to_owned());

    Ok(client.with_authentication(api_authentication))
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


    if seed_arguments.dry_run {
        println!("Dry run complete, exiting.");
        return Ok(());
    }


    // Insert all categories and note down their public UUIDs.
    let mut internal_to_public_category_id: HashMap<InternalCategoryId, CategoryId> = HashMap::new();

    let categories_api_client = authenticated_api_client.categories();

    println!();
    println!(
        "Inserting {} categories...",
        parsed_data.categories().values().len()
    );
    for parsed_category in parsed_data.categories().values() {
        println!(
            " > creating category: \"{}\"",
            parsed_category.english_name
        );

        let new_category = categories_api_client
            .create_category(CategoryToCreate {
                english_category_name: parsed_category.english_name.clone(),
                slovene_category_name: parsed_category.slovene_name.clone(),
                // Will be set in another pass.
                parent_category_id: None,
            })
            .send()
            .await
            .into_diagnostic()
            .wrap_err("failed to create category on the server")?;

        println!(
            "   | created with ID {}",
            new_category.id.into_uuid().as_hyphenated()
        );

        internal_to_public_category_id.insert(parsed_category.internal_id(), new_category.id);
    }

    println!();
    println!(
        "Updating {} categories' parents...",
        parsed_data.categories().values().len()
    );
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

        println!(
            " > setting parent of category \"{}\" ({}) to \"{}\" ({})",
            parsed_category.english_name,
            associated_real_category_id.into_uuid().as_hyphenated(),
            parent_category.english_name,
            associated_real_parent_category_id
                .into_uuid()
                .as_hyphenated()
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
            .into_diagnostic()
            .wrap_err("failed to set parent category, no changes")?
            .send()
            .await
            .into_diagnostic()
            .wrap_err("failed to set parent category on the server")?;

        println!("   | parent set");
    }


    // Insert all slovene words and their meanings and note down their public UUIDs.
    let mut internal_to_public_slovene_word_id: HashMap<InternalSloveneWordId, SloveneWordId> =
        HashMap::new();

    let mut internal_to_public_slovene_word_meaning_id: HashMap<
        InternalSloveneWordMeaningId,
        SloveneWordMeaningId,
    > = HashMap::new();


    let slovene_api_client = authenticated_api_client.slovene_dictionary();


    println!();
    println!(
        "Inserting {} slovene words...",
        parsed_data.slovene_words().values().len()
    );
    for slovene_word in parsed_data.slovene_words().values() {
        println!(
            " > creating slovene word: \"{}\"",
            slovene_word.lemma
        );

        let new_slovene_word = slovene_api_client
            .create_slovene_word(SloveneWordToCreate {
                lemma: slovene_word.lemma.clone(),
            })
            .send()
            .await
            .into_diagnostic()
            .wrap_err_with(|| {
                miette!(
                    "failed to create new slovene word (\"{}\")",
                    slovene_word.lemma
                )
            })?;

        println!(
            "  | created with ID {}",
            new_slovene_word.id.into_uuid().as_hyphenated()
        );

        internal_to_public_slovene_word_id.insert(slovene_word.internal_id(), new_slovene_word.id);
    }


    println!();
    println!(
        "Inserting {} slovene word meanings...",
        parsed_data.slovene_word_meanings().values().len()
    );
    for slovene_word_meaning in parsed_data.slovene_word_meanings().values() {
        println!(
            " > creating slovene word meaning: {} / {} / {} ",
            slovene_word_meaning
                .abbreviation
                .as_deref()
                .unwrap_or("None"),
            slovene_word_meaning
                .disambiguation
                .as_deref()
                .unwrap_or("None"),
            slovene_word_meaning
                .description
                .as_deref()
                .unwrap_or("None")
        );

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
            .send()
            .await
            .into_diagnostic()
            .wrap_err("failed to create new slovene word meaning")?;


        println!(
            "    | created with ID {}",
            new_slovene_word_meaning
                .word_meaning_id
                .into_uuid()
                .as_hyphenated()
        );

        internal_to_public_slovene_word_meaning_id.insert(
            slovene_word_meaning.internal_id(),
            new_slovene_word_meaning.word_meaning_id,
        );


        let connected_categories = slovene_word_meaning
            .categories(parsed_data.categories())
            .into_diagnostic()
            .wrap_err("expected to have already created all associated categories")?;

        println!(
            "    | linking with {} categories...",
            connected_categories.len()
        );

        for category in connected_categories {
            let target_category_id = internal_to_public_category_id
                .get(&category.internal_id())
                .ok_or_else(|| {
                    miette!("expected to have already created all associated categories")
                })?;

            println!(
                "    | linking with category \"{}\" ({})",
                category.english_name,
                target_category_id.into_uuid().as_hyphenated()
            );

            slovene_api_client
                .link_category_to_slovene_word_meaning(
                    *associated_slovene_word,
                    new_slovene_word_meaning.word_meaning_id,
                    *target_category_id,
                )
                .send()
                .await
                .into_diagnostic()
                .wrap_err_with(|| {
                    miette!(
                        "failed to link slovene word meaning {} with the category {}",
                        new_slovene_word_meaning.word_meaning_id,
                        target_category_id
                    )
                })?;

            println!("    | linked");
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


    println!();
    println!(
        "Inserting {} english words...",
        parsed_data.english_words().values().len()
    );
    for english_word in parsed_data.english_words().values() {
        println!(
            " > creating english word: \"{}\"",
            english_word.lemma
        );

        let new_english_word = english_api_client
            .create_english_word(EnglishWordToCreate {
                lemma: english_word.lemma.clone(),
            })
            .send()
            .await
            .into_diagnostic()
            .wrap_err("failed to create new english word")?;

        println!(
            "  | created with ID {}",
            new_english_word.id.into_uuid().as_hyphenated()
        );

        internal_to_public_english_word_id.insert(english_word.internal_id(), new_english_word.id);
    }


    println!();
    println!(
        "Inserting {} english word meanings...",
        parsed_data.english_word_meanings().values().len()
    );
    for english_word_meaning in parsed_data.english_word_meanings().values() {
        println!(
            " > creating english word meaning: {} / {} / {} ",
            english_word_meaning
                .abbreviation
                .as_deref()
                .unwrap_or("None"),
            english_word_meaning
                .disambiguation
                .as_deref()
                .unwrap_or("None"),
            english_word_meaning
                .description
                .as_deref()
                .unwrap_or("None")
        );

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
            .send()
            .await
            .into_diagnostic()
            .wrap_err("failed to create new english word meaning")?;


        println!(
            "    | created with ID {}",
            new_english_word_meaning
                .word_meaning_id
                .into_uuid()
                .as_hyphenated()
        );

        internal_to_public_english_word_meaning_id.insert(
            english_word_meaning.internal_id(),
            new_english_word_meaning.word_meaning_id,
        );


        let connected_categories = english_word_meaning
            .categories(parsed_data.categories())
            .into_diagnostic()
            .wrap_err("expected to have already created all associated categories")?;

        println!(
            "    | linking with {} categories...",
            connected_categories.len()
        );

        for category in connected_categories {
            let target_category_id = internal_to_public_category_id
                .get(&category.internal_id())
                .ok_or_else(|| {
                    miette!("expected to have already created all associated categories")
                })?;

            println!(
                "    | linking with category \"{}\" ({})",
                category.english_name,
                target_category_id.into_uuid().as_hyphenated()
            );

            english_api_client
                .link_category_to_english_word_meaning(
                    *associated_english_word,
                    new_english_word_meaning.word_meaning_id,
                    *target_category_id,
                )
                .send()
                .await
                .into_diagnostic()
                .wrap_err("failed to link english word meaning with a category")?;

            println!("    | linked");
        }
    }


    // Establish translation relationships between english and slovene word meanings.

    let translations_api_client = authenticated_api_client.translations();

    println!();
    println!(
        "Inserting {} translation relationships...",
        parsed_data.translations().values().len()
    );
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


        println!(
            " > creating translation between {} and {}",
            associated_slovene_word_meaning_id
                .into_uuid()
                .as_hyphenated(),
            associated_english_word_meaning_id
                .into_uuid()
                .as_hyphenated()
        );


        translations_api_client
            .create_translation_relationship(TranslationRelationshipToCreate {
                slovene_word_meaning: *associated_slovene_word_meaning_id,
                english_word_meaning: *associated_english_word_meaning_id,
            })
            .send()
            .await
            .into_diagnostic()
            .wrap_err("failed to create translation relationship")?;

        println!("    | translation relationship created");
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
