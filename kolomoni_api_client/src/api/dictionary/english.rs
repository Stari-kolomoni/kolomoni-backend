use chrono::{DateTime, Utc};
use kolomoni_core::{
    api_models::{
        EnglishWordCreationRequest,
        EnglishWordCreationResponse,
        EnglishWordInfoResponse,
        EnglishWordMeaning,
        EnglishWordMeaningUpdateRequest,
        EnglishWordMeaningUpdatedResponse,
        EnglishWordMeaningWithDetails,
        EnglishWordMeaningsResponse,
        EnglishWordUpdateRequest,
        EnglishWordWithMeanings,
        EnglishWordsResponse,
        NewEnglishWordMeaningCreatedResponse,
        NewEnglishWordMeaningRequest,
        WordErrorReason,
    },
    ids::{CategoryId, EnglishWordId, EnglishWordMeaningId},
};
use reqwest::StatusCode;
use thiserror::Error;

use crate::{
    api::EndpointGroup,
    client::{errors::RequestError, KolomoniHttpClient},
    parsing::err_if_invalid_uuid,
    request::typed::{BoundTypedRequest, IntoBoundTypedRequest},
    response::{raw::RawResponse, ResponseValueError},
};
use crate::{
    parsing::{err_if_missing_permissions, unexpected_error_reason, unexpected_response},
    request::ToRequestBuilder,
};



pub struct EnglishWordFetchingOptions {
    pub only_words_modified_after: Option<DateTime<Utc>>,
}


pub struct EnglishWordToCreate {
    pub lemma: String,
}


pub struct EnglishWordFieldsToUpdate {
    pub new_lemma: Option<String>,
}

impl EnglishWordFieldsToUpdate {
    pub(crate) fn has_no_fields_to_update(&self) -> bool {
        self.new_lemma.is_none()
    }
}


#[derive(Debug, Error)]
pub enum EnglishWordListFetchingError {
    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for EnglishWordListFetchingError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}


#[derive(Debug, Error)]
pub enum EnglishWordFetchingError {
    #[error("english word not found")]
    NotFound,

    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for EnglishWordFetchingError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}


#[derive(Debug, Error)]
pub enum EnglishWordCreationError {
    #[error("an english word with this lemma already exists")]
    LemmaAlreadyExists,

    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for EnglishWordCreationError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}



#[derive(Debug, Error)]
pub enum EnglishWordUpdatePreparationError {
    #[error("there was nothing to update")]
    NothingToUpdate,
}

#[derive(Debug, Error)]
pub enum EnglishWordUpdateError {
    #[error("english word not found")]
    NotFound,

    #[error("an english word with this lemma already exists")]
    LemmaAlreadyExists,

    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for EnglishWordUpdateError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}


#[derive(Debug, Error)]
pub enum EnglishWordDeletionError {
    #[error("english word not found")]
    NotFound,

    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for EnglishWordDeletionError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}


/*
 * English word-related endpoints
 */


fn get_english_words_request<'c, C>(
    client: &'c C,
    options: EnglishWordFetchingOptions,
) -> BoundTypedRequest<'c, C, Vec<EnglishWordWithMeanings>, EnglishWordListFetchingError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let words_response = response.into_json_body::<EnglishWordsResponse>().await?;

            Ok(words_response.english_words)
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            err_if_missing_permissions::<EnglishWordListFetchingError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else {
            Err(unexpected_response(response).await)
        }
    };


    let request_builder = if let Some(only_last_modified_after) = options.only_words_modified_after {
        client.get().endpoint_url_with_parameters(
            "/dictionary/english/words",
            [(
                "last_modified_after",
                only_last_modified_after.to_rfc3339(),
            )],
        )
    } else {
        client.get().endpoint_url("/dictionary/english")
    };

    request_builder
        .build_request()
        .into_bound_typed_request(response_parser)
}


fn get_english_word_by_id_request<'c, C>(
    client: &'c C,
    english_word_id: EnglishWordId,
) -> BoundTypedRequest<'c, C, EnglishWordWithMeanings, EnglishWordFetchingError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let response_body = response.into_json_body::<EnglishWordInfoResponse>().await?;

            Ok(response_body.word)
        } else if status == StatusCode::NOT_FOUND {
            let error_reason = response.word_error_reason().await?;

            match error_reason {
                WordErrorReason::WordNotFound => Err(EnglishWordFetchingError::NotFound),
                _ => Err(unexpected_error_reason(status, error_reason)),
            }
        } else if status == StatusCode::BAD_REQUEST {
            let error_reason = response.error_reason().await?;

            err_if_invalid_uuid::<EnglishWordFetchingError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .get()
        .endpoint_url(format!(
            "/dictionary/english/words/{}",
            english_word_id.into_uuid()
        ))
        .build_request()
        .into_bound_typed_request(response_parser)
}


fn get_english_word_by_lemma_request<'c, C>(
    client: &'c C,
    english_word_lemma: &str,
) -> BoundTypedRequest<'c, C, EnglishWordWithMeanings, EnglishWordFetchingError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let response_body = response.into_json_body::<EnglishWordInfoResponse>().await?;

            Ok(response_body.word)
        } else if status == StatusCode::NOT_FOUND {
            let word_error_reason = response.word_error_reason().await?;

            match word_error_reason {
                WordErrorReason::WordNotFound => Err(EnglishWordFetchingError::NotFound),
                _ => Err(unexpected_error_reason(status, word_error_reason)),
            }
        } else if status == StatusCode::BAD_REQUEST {
            let error_reason = response.error_reason().await?;

            err_if_invalid_uuid::<EnglishWordFetchingError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            err_if_missing_permissions::<EnglishWordFetchingError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .get()
        .endpoint_url(format!(
            "/dictionary/english/words/by-lemma/{}",
            english_word_lemma
        ))
        .build_request()
        .into_bound_typed_request(response_parser)
}


fn create_english_word_request<'c, C>(
    client: &'c C,
    new_word: EnglishWordToCreate,
) -> BoundTypedRequest<'c, C, EnglishWordWithMeanings, EnglishWordCreationError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let response_body = response
                .into_json_body::<EnglishWordCreationResponse>()
                .await?;

            Ok(response_body.word)
        } else if status == StatusCode::CONFLICT {
            let word_error_reason = response.word_error_reason().await?;

            match word_error_reason {
                WordErrorReason::WordWithThisLemmaAlreadyExists => {
                    Err(EnglishWordCreationError::LemmaAlreadyExists)
                }
                _ => Err(unexpected_error_reason(status, word_error_reason)),
            }
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            err_if_missing_permissions::<EnglishWordCreationError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .post()
        .endpoint_url("/dictionary/english/words")
        .json(&EnglishWordCreationRequest {
            lemma: new_word.lemma,
        })
        .build_request()
        .into_bound_typed_request(response_parser)
}


fn update_english_word_request<'c, C>(
    client: &'c C,
    english_word_id: EnglishWordId,
    fields_to_update: EnglishWordFieldsToUpdate,
) -> Result<
    BoundTypedRequest<'c, C, EnglishWordWithMeanings, EnglishWordUpdateError>,
    EnglishWordUpdatePreparationError,
>
where
    C: KolomoniHttpClient,
{
    if fields_to_update.has_no_fields_to_update() {
        return Err(EnglishWordUpdatePreparationError::NothingToUpdate);
    }

    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let response_body = response.into_json_body::<EnglishWordInfoResponse>().await?;

            Ok(response_body.word)
        } else if status == StatusCode::NOT_FOUND {
            let word_error_reason = response.word_error_reason().await?;

            match word_error_reason {
                WordErrorReason::WordNotFound => Err(EnglishWordUpdateError::NotFound),
                _ => Err(unexpected_error_reason(status, word_error_reason)),
            }
        } else if status == StatusCode::CONFLICT {
            let word_error_reason = response.word_error_reason().await?;

            match word_error_reason {
                WordErrorReason::WordWithThisLemmaAlreadyExists => {
                    Err(EnglishWordUpdateError::LemmaAlreadyExists)
                }
                _ => Err(unexpected_error_reason(status, word_error_reason)),
            }
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            err_if_missing_permissions::<EnglishWordUpdateError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else {
            Err(unexpected_response(response).await)
        }
    };

    Ok(client
        .patch()
        .endpoint_url(format!(
            "/dictionary/english/words/{}",
            english_word_id
        ))
        .json(&EnglishWordUpdateRequest {
            lemma: fields_to_update.new_lemma,
        })
        .build_request()
        .into_bound_typed_request(response_parser))
}


fn delete_english_word_request<'c, C>(
    client: &'c C,
    english_word_id: EnglishWordId,
) -> BoundTypedRequest<'c, C, (), EnglishWordDeletionError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            Ok(())
        } else if status == StatusCode::NOT_FOUND {
            let word_error_reason = response.word_error_reason().await?;

            match word_error_reason {
                WordErrorReason::WordNotFound => Err(EnglishWordDeletionError::NotFound),
                _ => Err(unexpected_error_reason(status, word_error_reason)),
            }
        } else if status == StatusCode::BAD_REQUEST {
            let error_reason = response.error_reason().await?;

            err_if_invalid_uuid::<EnglishWordDeletionError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            err_if_missing_permissions::<EnglishWordDeletionError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .delete()
        .endpoint_url(format!(
            "/dictionary/english/words/{}",
            english_word_id
        ))
        .build_request()
        .into_bound_typed_request(response_parser)
}


/*
 * English word meaning-related endpoints
 */

pub struct EnglishWordMeaningToCreate {
    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,
}


pub struct EnglishWordMeaningFieldsToUpdate {
    pub disambiguation: Option<Option<String>>,

    pub abbreviation: Option<Option<String>>,

    pub description: Option<Option<String>>,
}

impl EnglishWordMeaningFieldsToUpdate {
    pub(crate) fn has_no_fields_to_update(&self) -> bool {
        self.disambiguation.is_none() && self.abbreviation.is_none() && self.description.is_none()
    }
}



#[derive(Debug, Error)]
pub enum EnglishWordMeaningsFetchingError {
    #[error("english word does not exist")]
    WordNotFound,

    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for EnglishWordMeaningsFetchingError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}


#[derive(Debug, Error)]
pub enum EnglishWordMeaningCreationError {
    #[error("english word does not exist")]
    WordNotFound,

    #[error("identical english word meaning already exists")]
    IdenticalWordMeaningAlreadyExists,

    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for EnglishWordMeaningCreationError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}


#[derive(Debug, Error)]
pub enum EnglishWordMeaningUpdatePreparationError {
    #[error("there were no fields to update")]
    NothingToUpdate,
}

#[derive(Debug, Error)]
pub enum EnglishWordMeaningUpdateError {
    #[error("english word does not exist")]
    WordNotFound,

    #[error("english word meaning does not exist")]
    WordMeaningNotFound,

    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for EnglishWordMeaningUpdateError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}


#[derive(Debug, Error)]
pub enum EnglishWordMeaningDeletionError {
    #[error("english word does not exist")]
    WordNotFound,

    #[error("english word meaning does not exist")]
    WordMeaningNotFound,

    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for EnglishWordMeaningDeletionError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}


#[derive(Debug, Error)]
pub enum EnglishWordMeaningCategoryLinkingError {
    #[error("english word does not exist")]
    WordNotFound,

    #[error("english word meaning does not exist")]
    WordMeaningNotFound,

    #[error("english word meaning is already linked to the given category")]
    CategoryRelationshipAlreadyExists,

    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for EnglishWordMeaningCategoryLinkingError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}

#[derive(Debug, Error)]
pub enum EnglishWordMeaningCategoryUnlinkingError {
    #[error("english word does not exist")]
    WordNotFound,

    #[error("english word meaning does not exist")]
    WordMeaningNotFound,

    #[error("english word meaning-category relationship does not exist")]
    CategoryRelationshipNotFound,

    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for EnglishWordMeaningCategoryUnlinkingError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}



fn get_english_word_meanings_request<'c, C>(
    client: &'c C,
    english_word_id: EnglishWordId,
) -> BoundTypedRequest<'c, C, Vec<EnglishWordMeaningWithDetails>, EnglishWordMeaningsFetchingError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let response_body = response
                .into_json_body::<EnglishWordMeaningsResponse>()
                .await?;

            Ok(response_body.meanings)
        } else if status == StatusCode::NOT_FOUND {
            let word_error_response = response.word_error_reason().await?;

            match word_error_response {
                WordErrorReason::WordNotFound => Err(EnglishWordMeaningsFetchingError::WordNotFound),
                _ => Err(unexpected_error_reason(
                    status,
                    word_error_response,
                )),
            }
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            err_if_missing_permissions::<EnglishWordMeaningsFetchingError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .get()
        .endpoint_url(format!(
            "/dictionary/english/words/{}/meanings",
            english_word_id
        ))
        .build_request()
        .into_bound_typed_request(response_parser)
}


fn create_english_word_meaning_request<'c, C>(
    client: &'c C,
    english_word_id: EnglishWordId,
    meaning: EnglishWordMeaningToCreate,
) -> BoundTypedRequest<'c, C, EnglishWordMeaning, EnglishWordMeaningCreationError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let response_body = response
                .into_json_body::<NewEnglishWordMeaningCreatedResponse>()
                .await?;

            Ok(response_body.meaning)
        } else if status == StatusCode::CONFLICT {
            let word_error_reason = response.word_error_reason().await?;

            match word_error_reason {
                WordErrorReason::IdenticalWordMeaningAlreadyExists => {
                    Err(EnglishWordMeaningCreationError::IdenticalWordMeaningAlreadyExists)
                }
                _ => Err(unexpected_error_reason(status, word_error_reason)),
            }
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            err_if_missing_permissions::<EnglishWordMeaningCreationError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .post()
        .endpoint_url(format!(
            "/dictionary/english/words/{}/meanings",
            english_word_id
        ))
        .json(&NewEnglishWordMeaningRequest {
            abbreviation: meaning.abbreviation,
            disambiguation: meaning.disambiguation,
            description: meaning.description,
        })
        .build_request()
        .into_bound_typed_request(response_parser)
}


fn update_english_word_meaning_request<'c, C>(
    client: &'c C,
    english_word_id: EnglishWordId,
    english_word_meaning_id: EnglishWordMeaningId,
    fields_to_update: EnglishWordMeaningFieldsToUpdate,
) -> Result<
    BoundTypedRequest<'c, C, EnglishWordMeaningWithDetails, EnglishWordMeaningUpdateError>,
    EnglishWordMeaningUpdatePreparationError,
>
where
    C: KolomoniHttpClient,
{
    if fields_to_update.has_no_fields_to_update() {
        return Err(EnglishWordMeaningUpdatePreparationError::NothingToUpdate);
    }


    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let response_body = response
                .into_json_body::<EnglishWordMeaningUpdatedResponse>()
                .await?;

            Ok(response_body.meaning)
        } else if status == StatusCode::NOT_FOUND {
            let word_error_reason = response.word_error_reason().await?;

            match word_error_reason {
                WordErrorReason::WordNotFound => Err(EnglishWordMeaningUpdateError::WordNotFound),
                WordErrorReason::WordMeaningNotFound => {
                    Err(EnglishWordMeaningUpdateError::WordMeaningNotFound)
                }
                _ => Err(unexpected_error_reason(status, word_error_reason)),
            }
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            err_if_missing_permissions::<EnglishWordMeaningUpdateError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else {
            Err(unexpected_response(response).await)
        }
    };

    Ok(client
        .patch()
        .endpoint_url(format!(
            "/dictionary/english/words/{}/meanings/{}",
            english_word_id, english_word_meaning_id
        ))
        .json(&EnglishWordMeaningUpdateRequest {
            abbreviation: fields_to_update.abbreviation,
            disambiguation: fields_to_update.disambiguation,
            description: fields_to_update.description,
        })
        .build_request()
        .into_bound_typed_request(response_parser))
}


fn delete_english_word_meaning_request<'c, C>(
    client: &'c C,
    english_word_id: EnglishWordId,
    english_word_meaning_id: EnglishWordMeaningId,
) -> BoundTypedRequest<'c, C, (), EnglishWordMeaningDeletionError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            Ok(())
        } else if status == StatusCode::NOT_FOUND {
            let word_error_reason = response.word_error_reason().await?;

            match word_error_reason {
                WordErrorReason::WordNotFound => Err(EnglishWordMeaningDeletionError::WordNotFound),
                WordErrorReason::WordMeaningNotFound => {
                    Err(EnglishWordMeaningDeletionError::WordMeaningNotFound)
                }
                _ => Err(unexpected_error_reason(status, word_error_reason)),
            }
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            err_if_missing_permissions::<EnglishWordMeaningDeletionError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .delete()
        .endpoint_url(format!(
            "/dictionary/english/words/{}/meanings/{}",
            english_word_id, english_word_meaning_id
        ))
        .build_request()
        .into_bound_typed_request(response_parser)
}


fn link_category_to_english_word_meaning_request<'c, C>(
    client: &'c C,
    english_word_id: EnglishWordId,
    english_word_meaning_id: EnglishWordMeaningId,
    category_id: CategoryId,
) -> BoundTypedRequest<'c, C, (), EnglishWordMeaningCategoryLinkingError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            Ok(())
        } else if status == StatusCode::NOT_FOUND {
            let word_error_reason = response.word_error_reason().await?;

            match word_error_reason {
                WordErrorReason::WordNotFound => {
                    Err(EnglishWordMeaningCategoryLinkingError::WordNotFound)
                }
                WordErrorReason::WordMeaningNotFound => {
                    Err(EnglishWordMeaningCategoryLinkingError::WordMeaningNotFound)
                }
                _ => Err(unexpected_error_reason(status, word_error_reason)),
            }
        } else if status == StatusCode::CONFLICT {
            let word_error_reason = response.word_error_reason().await?;

            match word_error_reason {
                WordErrorReason::WordMeaningAlreadyHasThisCategory => {
                    Err(EnglishWordMeaningCategoryLinkingError::CategoryRelationshipAlreadyExists)
                }
                _ => Err(unexpected_error_reason(status, word_error_reason)),
            }
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            err_if_missing_permissions::<EnglishWordMeaningCategoryLinkingError>(
                status,
                &error_reason,
            )?;
            Err(unexpected_error_reason(status, error_reason))
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .post()
        .endpoint_url(format!(
            "/dictionary/english/words/{}/meanings/{}/categories/{}",
            english_word_id, english_word_meaning_id, category_id
        ))
        .build_request()
        .into_bound_typed_request(response_parser)
}


fn unlink_category_from_english_word_meaning_request<'c, C>(
    client: &'c C,
    english_word_id: EnglishWordId,
    english_word_meaning_id: EnglishWordMeaningId,
    category_id: CategoryId,
) -> BoundTypedRequest<'c, C, (), EnglishWordMeaningCategoryUnlinkingError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            Ok(())
        } else if status == StatusCode::NOT_FOUND {
            let word_error_reason = response.word_error_reason().await?;

            match word_error_reason {
                WordErrorReason::WordNotFound => {
                    Err(EnglishWordMeaningCategoryUnlinkingError::WordNotFound)
                }
                WordErrorReason::WordMeaningNotFound => {
                    Err(EnglishWordMeaningCategoryUnlinkingError::WordMeaningNotFound)
                }
                WordErrorReason::WordMeaningCategoryRelationshipNotFound => {
                    Err(EnglishWordMeaningCategoryUnlinkingError::CategoryRelationshipNotFound)
                }
                _ => Err(unexpected_error_reason(status, word_error_reason)),
            }
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            err_if_missing_permissions::<EnglishWordMeaningCategoryUnlinkingError>(
                status,
                &error_reason,
            )?;
            Err(unexpected_error_reason(status, error_reason))
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .delete()
        .endpoint_url(format!(
            "/dictionary/english/words/{}/meanings/{}/categories/{}",
            english_word_id, english_word_meaning_id, category_id
        ))
        .build_request()
        .into_bound_typed_request(response_parser)
}



pub trait EnglishDictionaryUnauthenticatedEndpoints<'c, C>: EndpointGroup<'c, C>
where
    C: KolomoniHttpClient,
{
    /*
     * Word-related (word meanings are in the next section)
     */
    fn english_words(
        &'c self,
        options: EnglishWordFetchingOptions,
    ) -> BoundTypedRequest<'c, C, Vec<EnglishWordWithMeanings>, EnglishWordListFetchingError> {
        get_english_words_request(self.client(), options)
    }

    fn english_word_by_id(
        &'c self,
        english_word_id: EnglishWordId,
    ) -> BoundTypedRequest<'c, C, EnglishWordWithMeanings, EnglishWordFetchingError> {
        get_english_word_by_id_request(self.client(), english_word_id)
    }

    fn english_word_by_lemma(
        &'c self,
        english_word_lemma: &str,
    ) -> BoundTypedRequest<'c, C, EnglishWordWithMeanings, EnglishWordFetchingError> {
        get_english_word_by_lemma_request(self.client(), english_word_lemma)
    }

    /*
     * Word meaning-related (words themselves are in the previous section)
     */
    fn english_word_meanings(
        &'c self,
        english_word_id: EnglishWordId,
    ) -> BoundTypedRequest<'c, C, Vec<EnglishWordMeaningWithDetails>, EnglishWordMeaningsFetchingError>
    {
        get_english_word_meanings_request(self.client(), english_word_id)
    }
}


pub trait EnglishDictionaryAuthenticatedEndpoints<'c, C>: EndpointGroup<'c, C>
where
    C: KolomoniHttpClient,
{
    /*
     * Word-related (word meanings are in the next section)
     */
    fn create_english_word(
        &'c self,
        word: EnglishWordToCreate,
    ) -> BoundTypedRequest<'c, C, EnglishWordWithMeanings, EnglishWordCreationError> {
        create_english_word_request(self.client(), word)
    }

    fn update_english_word(
        &'c self,
        english_word_id: EnglishWordId,
        fields_to_update: EnglishWordFieldsToUpdate,
    ) -> Result<
        BoundTypedRequest<'c, C, EnglishWordWithMeanings, EnglishWordUpdateError>,
        EnglishWordUpdatePreparationError,
    > {
        update_english_word_request(self.client(), english_word_id, fields_to_update)
    }

    fn delete_english_word(
        &'c self,
        english_word_id: EnglishWordId,
    ) -> BoundTypedRequest<'c, C, (), EnglishWordDeletionError> {
        delete_english_word_request(self.client(), english_word_id)
    }


    /*
     * Word meaning-related (words themselves are in the previous section)
     */
    fn create_english_word_meaning(
        &'c self,
        english_word_id: EnglishWordId,
        meaning: EnglishWordMeaningToCreate,
    ) -> BoundTypedRequest<'c, C, EnglishWordMeaning, EnglishWordMeaningCreationError> {
        create_english_word_meaning_request(self.client(), english_word_id, meaning)
    }

    fn update_english_word_meaning(
        &'c self,
        english_word_id: EnglishWordId,
        english_word_meaning_id: EnglishWordMeaningId,
        fields_to_update: EnglishWordMeaningFieldsToUpdate,
    ) -> Result<
        BoundTypedRequest<'c, C, EnglishWordMeaningWithDetails, EnglishWordMeaningUpdateError>,
        EnglishWordMeaningUpdatePreparationError,
    > {
        update_english_word_meaning_request(
            self.client(),
            english_word_id,
            english_word_meaning_id,
            fields_to_update,
        )
    }

    fn delete_english_word_meaning(
        &'c self,
        english_word_id: EnglishWordId,
        english_word_meaning_id: EnglishWordMeaningId,
    ) -> BoundTypedRequest<'c, C, (), EnglishWordMeaningDeletionError> {
        delete_english_word_meaning_request(
            self.client(),
            english_word_id,
            english_word_meaning_id,
        )
    }

    fn link_category_to_english_word_meaning(
        &'c self,
        english_word_id: EnglishWordId,
        english_word_meaning_id: EnglishWordMeaningId,
        category_id: CategoryId,
    ) -> BoundTypedRequest<'c, C, (), EnglishWordMeaningCategoryLinkingError> {
        link_category_to_english_word_meaning_request(
            self.client(),
            english_word_id,
            english_word_meaning_id,
            category_id,
        )
    }

    fn unlink_category_from_english_word_meaning(
        &'c self,
        english_word_id: EnglishWordId,
        english_word_meaning_id: EnglishWordMeaningId,
        category_id: CategoryId,
    ) -> BoundTypedRequest<'c, C, (), EnglishWordMeaningCategoryUnlinkingError> {
        unlink_category_from_english_word_meaning_request(
            self.client(),
            english_word_id,
            english_word_meaning_id,
            category_id,
        )
    }
}



pub struct EnglishDictionaryUnauthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
    client: &'c C,
}

impl<'c, C> EnglishDictionaryUnauthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
    pub(crate) const fn new(client: &'c C) -> Self {
        Self { client }
    }
}

impl<'c, C> EndpointGroup<'c, C> for EnglishDictionaryUnauthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
    fn client(&'c self) -> &'c C {
        self.client
    }
}

impl<'c, C> EnglishDictionaryUnauthenticatedEndpoints<'c, C>
    for EnglishDictionaryUnauthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
}




pub struct EnglishDictionaryAuthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
    client: &'c C,
}

impl<'c, C> EnglishDictionaryAuthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
    pub(crate) const fn new(client: &'c C) -> Self {
        Self { client }
    }
}

impl<'c, C> EndpointGroup<'c, C> for EnglishDictionaryAuthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
    fn client(&'c self) -> &'c C {
        self.client
    }
}

impl<'c, C> EnglishDictionaryUnauthenticatedEndpoints<'c, C>
    for EnglishDictionaryAuthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
}

impl<'c, C> EnglishDictionaryAuthenticatedEndpoints<'c, C>
    for EnglishDictionaryAuthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
}
