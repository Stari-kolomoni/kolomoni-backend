use std::collections::{hash_map::Entry, HashMap};

use actix_web::{post, web, Scope};
use itertools::Itertools;
use kolomoni_cache::{EnglishWordMeaningsLookupError, EntityCache};
use kolomoni_core::{
    api_models::{
        EnglishTranslation,
        EnglishWordMeaningWithDetails,
        ScoredEnglishWordMeaningWithDetails,
        ScoredSloveneWordMeaningWithDetails,
        SearchRequest,
        SearchResponse,
        SloveneTranslation,
        SloveneWordMeaningWithDetails,
        WordSearchResult,
    },
    ids::{CategoryId, EnglishWordMeaningId, SloveneWordMeaningId, UserId, WordId, WordMeaningId},
};
use kolomoni_search::{SearchResults, WordMeaningSearchResult};
use thiserror::Error;

use crate::{
    api::{
        errors::{EndpointError, EndpointResponseBuilder, EndpointResult},
        openapi,
        traits::IntoApiModel,
    },
    state::ApplicationState,
};


#[derive(Debug, Error)]
pub enum DetailedSearchResultGenerationError {
    /// This may happen in edge cases where an index hasn't been updated
    /// after removing a word from the database in time for the search.
    /// This should be rare, if not impossible.
    #[error("unable to find matched word in cache: {}", .word_id)]
    MatchedWordNotFoundInCache { word_id: WordId },

    /// This may happen in edge cases where an index hasn't been updated
    /// after removing a word / meaning from the database in time for the search.
    /// This should be rare, if not impossible.
    #[error("unable to find matched word meaning in cache: {}", .word_meaning_id)]
    MatchedWordMeaningNotFoundInCache { word_meaning_id: WordMeaningId },
}

impl From<EnglishWordMeaningsLookupError> for DetailedSearchResultGenerationError {
    fn from(value: EnglishWordMeaningsLookupError) -> Self {
        match value {
            EnglishWordMeaningsLookupError::WordMeaningNotFoundInCache { word_meaning_id } => {
                Self::MatchedWordMeaningNotFoundInCache {
                    word_meaning_id: word_meaning_id.upcast_to_word_meaning_id(),
                }
            }
        }
    }
}


fn build_scored_english_word_meaning_from_cache(
    english_word_meaning_id: EnglishWordMeaningId,
    result_score: f32,
    entity_cache: &EntityCache,
) -> Option<ScoredEnglishWordMeaningWithDetails> {
    // We fetch the cached english word meaning, ignoring any results that are not in cache yet.
    // This is okay enough if the cache is diligently updated, which it should be
    // (and we don't care about inaccuracies in the incredibly small interval between updating the database and our cache).
    let cached_english_word_meaning = entity_cache.english_word_meaning(&english_word_meaning_id)?;

    let (meaning_model, bare_meaning_model) = cached_english_word_meaning
        .word_meaning()
        .to_owned()
        .into_inner();

    let categories: Vec<CategoryId> = cached_english_word_meaning
        .categories()
        .iter()
        .copied()
        .collect::<Vec<_>>();

    let translations: Vec<SloveneTranslation> = cached_english_word_meaning
        .translations()
        .iter()
        .filter_map(|slovene_word_meaning_id| {
            let translation = entity_cache.translation_by_id(
                cached_english_word_meaning.word_meaning().id(),
                *slovene_word_meaning_id,
            )?;

            let slovene_word_meaning = entity_cache.slovene_word_meaning(slovene_word_meaning_id)?;
            let slovene_word =
                entity_cache.slovene_word(slovene_word_meaning.word_meaning().parent_word_id())?;

            Some(SloveneTranslation {
                word: slovene_word.word().to_owned().into_api_model(),
                word_meaning: slovene_word_meaning
                    .word_meaning()
                    .to_owned()
                    .into_api_model(),
                translated_at: translation.translated_at().to_owned(),
                translated_by: translation.translated_by().map(UserId::to_owned),
            })
        })
        .collect();

    let english_word_meaning = EnglishWordMeaningWithDetails {
        // SAFETY: We obtained an english word meaning and destructed it into components manually.
        word_meaning_id: meaning_model
            .word_meaning_id
            .downcast_to_english_word_meaning_id_unchecked(),
        disambiguation: bare_meaning_model.disambiguation,
        abbreviation: bare_meaning_model.abbreviation,
        description: bare_meaning_model.description,
        created_at: meaning_model.created_at,
        last_modified_at: meaning_model.last_modified_at,
        categories,
        translations,
    };

    Some(ScoredEnglishWordMeaningWithDetails {
        score: result_score,
        english_word_meaning,
    })
}

fn build_scored_slovene_word_meaning_from_cache(
    slovene_word_meaning_id: SloveneWordMeaningId,
    result_score: f32,
    entity_cache: &EntityCache,
) -> Option<ScoredSloveneWordMeaningWithDetails> {
    // We fetch the cached slovene word meaning, ignoring any results that are not in cache yet.
    // This is okay enough if the cache is diligently updated, which it should be
    // (and we don't care about inaccuracies in the incredibly small interval between updating the database and our cache).
    let cached_slovene_word_meaning = entity_cache.slovene_word_meaning(&slovene_word_meaning_id)?;

    let (meaning_model, bare_meaning_model) = cached_slovene_word_meaning
        .word_meaning()
        .to_owned()
        .into_inner();

    let categories: Vec<CategoryId> = cached_slovene_word_meaning
        .categories()
        .iter()
        .copied()
        .collect::<Vec<_>>();

    let translations: Vec<EnglishTranslation> = cached_slovene_word_meaning
        .translations()
        .iter()
        .filter_map(|english_word_meaning_id| {
            let translation = entity_cache.translation_by_id(
                *english_word_meaning_id,
                cached_slovene_word_meaning.word_meaning().id(),
            )?;

            let english_word_meaning = entity_cache.english_word_meaning(english_word_meaning_id)?;
            let english_word =
                entity_cache.english_word(english_word_meaning.word_meaning().parent_word_id())?;

            Some(EnglishTranslation {
                word: english_word.word().to_owned().into_api_model(),
                word_meaning: english_word_meaning
                    .word_meaning()
                    .to_owned()
                    .into_api_model(),
                translated_at: translation.translated_at().to_owned(),
                translated_by: translation.translated_by().map(UserId::to_owned),
            })
        })
        .collect();

    let slovene_word_meaning = SloveneWordMeaningWithDetails {
        // SAFETY: We obtained a Slovene word meaning and destructed it into components manually.
        word_meaning_id: meaning_model
            .word_meaning_id
            .downcast_to_slovene_word_meaning_id_unchecked(),
        disambiguation: bare_meaning_model.disambiguation,
        abbreviation: bare_meaning_model.abbreviation,
        description: bare_meaning_model.description,
        created_at: meaning_model.created_at,
        last_modified_at: meaning_model.last_modified_at,
        categories,
        translations,
    };

    Some(ScoredSloveneWordMeaningWithDetails {
        score: result_score,
        slovene_word_meaning,
    })
}


/// Generates a full array of matching words (and their meanings),
/// sorted by descending match score.
fn generate_detailed_sorted_search_results(
    search_results: SearchResults,
    entity_cache: &EntityCache,
) -> Result<Vec<WordSearchResult>, DetailedSearchResultGenerationError> {
    let mut search_results_by_id: HashMap<WordId, WordSearchResult> =
        HashMap::with_capacity(search_results.word_meanings.len() / 2);

    for word_meaning in search_results.word_meanings {
        match word_meaning {
            WordMeaningSearchResult::English {
                result_score,
                word_id,
                word_meaning_id,
            } => {
                match search_results_by_id.entry(word_id.upcast_to_word_id()) {
                    Entry::Occupied(mut occupied_entry) => {
                        // In this case, the underlying word of this matched meaning has appeared
                        // before in the search results, so we simply merge this meaning into
                        // the existing [`WordSearchResult`].

                        let WordSearchResult::English {
                            cumulative_score,
                            word_meanings,
                            ..
                        } = occupied_entry.get_mut()
                        else {
                            // SAFETY: Word IDs are unique, and are therefore never able to insert or fetch
                            // a `WordSearchResult::Slovene` given a word ID that belongs to an english word.
                            //
                            // A panic here would indicate that we've lost a critical invariant:
                            // the uniqueness of IDs in our database.
                            //
                            // For more context, see the `vacant_entry.insert` call earlier in this branch.
                            panic!(
                                "BUG: observed WordSearchResult::Slovene under english word ID?!"
                            );
                        };

                        *cumulative_score += result_score;

                        let Some(scored_english_meaning) =
                            build_scored_english_word_meaning_from_cache(
                                word_meaning_id,
                                result_score,
                                entity_cache,
                            )
                        else {
                            return Err(DetailedSearchResultGenerationError::MatchedWordMeaningNotFoundInCache { word_meaning_id: word_meaning_id.upcast_to_word_meaning_id() });
                        };

                        word_meanings.push(scored_english_meaning);
                    }
                    Entry::Vacant(vacant_entry) => {
                        // In this case, the underlying word of this matched meaning has not appeared before
                        // in these search results, so we create a new [`WordSearchResult`].

                        let Some(english_word) = entity_cache.english_word(word_id) else {
                            return Err(
                                DetailedSearchResultGenerationError::MatchedWordNotFoundInCache {
                                    word_id: word_id.upcast_to_word_id(),
                                },
                            );
                        };

                        let Some(scored_english_meaning) =
                            build_scored_english_word_meaning_from_cache(
                                word_meaning_id,
                                result_score,
                                entity_cache,
                            )
                        else {
                            return Err(DetailedSearchResultGenerationError::MatchedWordMeaningNotFoundInCache { word_meaning_id: word_meaning_id.upcast_to_word_meaning_id() });
                        };

                        vacant_entry.insert(WordSearchResult::English {
                            cumulative_score: result_score,
                            word: english_word.word().to_owned().into_api_model(),
                            word_meanings: vec![scored_english_meaning],
                        });
                    }
                };
            }
            WordMeaningSearchResult::Slovene {
                result_score,
                word_id,
                word_meaning_id,
            } => {
                match search_results_by_id.entry(word_id.upcast_to_word_id()) {
                    Entry::Occupied(mut occupied_entry) => {
                        // In this case, the underlying word of this matched meaning has appeared
                        // before in the search results, so we simply merge this meaning into
                        // the existing [`WordSearchResult`].

                        let WordSearchResult::Slovene {
                            cumulative_score,
                            word_meanings,
                            ..
                        } = occupied_entry.get_mut()
                        else {
                            // SAFETY: Word IDs are unique, and are therefore never able to insert or fetch
                            // a `WordSearchResult::English` given a word ID that belongs to a Slovene word.
                            //
                            // A panic here would indicate that we've lost a critical invariant:
                            // the uniqueness of IDs in our database.
                            //
                            // For more context, see the `vacant_entry.insert` call earlier in this branch.
                            panic!(
                                "BUG: observed WordSearchResult::English under Slovene word ID?!"
                            );
                        };

                        *cumulative_score += result_score;

                        let Some(scored_slovene_meaning) =
                            build_scored_slovene_word_meaning_from_cache(
                                word_meaning_id,
                                result_score,
                                entity_cache,
                            )
                        else {
                            return Err(
                                DetailedSearchResultGenerationError::MatchedWordMeaningNotFoundInCache { word_meaning_id: word_meaning_id.upcast_to_word_meaning_id() }
                            );
                        };

                        word_meanings.push(scored_slovene_meaning);
                    }
                    Entry::Vacant(vacant_entry) => {
                        // In this case, the underlying word of this matched meaning has not appeared before
                        // in these search results, so we create a new [`WordSearchResult`].

                        let Some(slovene_word) = entity_cache.slovene_word(word_id) else {
                            return Err(
                                DetailedSearchResultGenerationError::MatchedWordNotFoundInCache {
                                    word_id: word_id.upcast_to_word_id(),
                                },
                            );
                        };

                        let Some(scored_slovene_meaning) =
                            build_scored_slovene_word_meaning_from_cache(
                                word_meaning_id,
                                result_score,
                                entity_cache,
                            )
                        else {
                            return Err(DetailedSearchResultGenerationError::MatchedWordMeaningNotFoundInCache { word_meaning_id: word_meaning_id.upcast_to_word_meaning_id() });
                        };

                        vacant_entry.insert(WordSearchResult::Slovene {
                            cumulative_score: result_score,
                            word: slovene_word.word().to_owned().into_api_model(),
                            word_meanings: vec![scored_slovene_meaning],
                        });
                    }
                };
            }
        }
    }

    // Resort word results by their cumulative score (descending).
    let sorted_results = search_results_by_id
        .into_iter()
        .sorted_unstable_by(
            |(_, first_search_result), (_, second_search_result)| {
                let first_cumulative_score = first_search_result.cumulative_score();
                let second_cumulative_score = second_search_result.cumulative_score();

                first_cumulative_score
                    .total_cmp(&second_cumulative_score)
                    .reverse()
            },
        )
        .map(|(_, search_result)| search_result)
        .collect::<Vec<_>>();

    Ok(sorted_results)
}


/// Search the dictionary
///
/// This endpoint performs a fuzzy search across the entire dictionary
/// and returns a list of english and slovene word results.
///
/// # Authentication
/// Authentication is not required on this endpoint.
#[utoipa::path(
    post,
    path = "/dictionary/search",
    tag = "dictionary:search",
    request_body(
        content = SearchRequest
    ),
    responses(
        (
            status = 200,
            description = "Search results.",
            body = SearchResponse
        ),
        openapi::response::RequiredJsonBodyErrors,
        openapi::response::InternalServerError,
    )
)]
#[post("")]
pub async fn perform_search(
    state: ApplicationState,
    request_body: web::Json<SearchRequest>,
) -> EndpointResult {
    // TODO Maybe create a new word.search permission and grant it to everyone?
    // TODO We'll probably need some rate limiting, especially this endpoint.

    let search_query = request_body.into_inner().search_query;

    let search_results = state
        .search_engine()
        .search(&search_query)
        .map_err(EndpointError::internal_error)?;

    let Ok(detailed_search_results) =
        generate_detailed_sorted_search_results(search_results, &state.cache_read())
    else {
        return EndpointResponseBuilder::internal_server_error().build();
    };

    EndpointResponseBuilder::ok()
        .with_json_body(SearchResponse {
            search_results: detailed_search_results,
        })
        .build()
}


#[rustfmt::skip]
pub fn search_router() -> Scope {
    web::scope("/search")
        .service(perform_search)
}
