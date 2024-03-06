use actix_web::{post, web, Scope};
use kolomoni_search::SearchResult;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{english_word::EnglishWord, slovene_word::SloveneWord};
use crate::{
    api::{
        errors::{APIError, EndpointResult},
        macros::ContextlessResponder,
        openapi,
    },
    impl_json_response_builder,
    state::ApplicationState,
};



#[derive(Deserialize, Clone, PartialEq, Eq, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Serialize))]
#[schema(
    example = json!({
        "search_query": "hit points"
    })
)]
pub struct SearchRequest {
    /// Search query.
    pub search_query: String,
}


#[derive(Serialize, Clone, PartialEq, Eq, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Deserialize))]
pub struct SearchResults {
    english_results: Vec<EnglishWord>,
    slovene_results: Vec<SloveneWord>,
}

#[derive(Serialize, Clone, PartialEq, Eq, ToSchema)]
#[cfg_attr(feature = "with_test_facilities", derive(Deserialize))]
#[schema(
    example = json!({
        "search_results": {
            "english_results": [],
            "slovene_results": [
                {
                    "id": "018def26-7a7a-73d5-9885-cfbaee7ce955",
                    "lemma": "terna",
                    "disambiguation": null,
                    "description": "Živjo svet!",
                    "created_at": "2024-02-28T09:58:12.858681Z",
                    "last_modified_at": "2024-02-28T09:58:12.863905Z",
                    "categories": []
                }
            ]
        }
    })
)]
pub struct SearchResponse {
    search_results: SearchResults,
}

impl_json_response_builder!(SearchResponse);


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
        openapi::MissingOrInvalidJsonRequestBodyResponse,
        openapi::InternalServerErrorResponse
    )
)]
#[post("")]
pub async fn perform_search(
    state: ApplicationState,
    request_body: web::Json<SearchRequest>,
) -> EndpointResult {
    // TODO Maybe create a new word.search permission and grant it to everyone?

    // TODO We'll probbaly need rate limiting, especially this endpoint.

    let search_query = request_body.into_inner().search_query;

    let search_results = state
        .search
        .search(&search_query)
        .await
        .map_err(APIError::InternalError)?;


    let mut english_results: Vec<EnglishWord> = Vec::new();
    let mut slovene_results: Vec<SloveneWord> = Vec::new();

    for search_result in search_results.words {
        match search_result {
            SearchResult::English(english_result) => {
                english_results.push(EnglishWord::from_expanded_word_info(
                    english_result,
                ));
            }
            SearchResult::Slovene(slovene_result) => {
                slovene_results.push(SloveneWord::from_expanded_word_info(
                    slovene_result,
                ));
            }
        }
    }


    Ok(SearchResponse {
        search_results: SearchResults {
            english_results,
            slovene_results,
        },
    }
    .into_response())
}


#[rustfmt::skip]
pub fn search_router() -> Scope {
    web::scope("/search")
        .service(perform_search)
}
