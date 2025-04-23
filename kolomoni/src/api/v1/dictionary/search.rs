use actix_web::{post, web, Scope};
use kolomoni_core::api_models::{SearchRequest, SearchResponse, SearchedWordMeaning};
use kolomoni_search::WordMeaningSearchResult;

use crate::{
    api::{
        errors::{EndpointError, EndpointResponseBuilder, EndpointResult},
        openapi,
        traits::IntoApiModel,
    },
    state::ApplicationState,
};


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



    let mut api_search_results = Vec::new();

    for search_result in search_results.word_meanings {
        match search_result {
            WordMeaningSearchResult::English {
                result_score,
                word,
                word_meaning,
            } => {
                let english_word = word.into_api_model();
                let english_word_meaning = word_meaning.into_api_model();

                api_search_results.push(SearchedWordMeaning::English {
                    result_score,
                    word: english_word,
                    word_meaning: english_word_meaning,
                });
            }
            WordMeaningSearchResult::Slovene {
                result_score,
                word,
                word_meaning,
            } => {
                let slovene_word = word.into_api_model();
                let slovene_word_meaning = word_meaning.into_api_model();

                api_search_results.push(SearchedWordMeaning::Slovene {
                    result_score,
                    word: slovene_word,
                    word_meaning: slovene_word_meaning,
                });
            }
        }
    }

    EndpointResponseBuilder::ok()
        .with_json_body(SearchResponse {
            word_meanings: api_search_results,
        })
        .build()
}


#[rustfmt::skip]
pub fn search_router() -> Scope {
    web::scope("/search")
        .service(perform_search)
}
