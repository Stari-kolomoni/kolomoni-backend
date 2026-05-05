use kolomoni_core::{
    api_models::{
        NewSloveneWordMeaningCreatedResponse,
        NewSloveneWordMeaningRequest,
        SloveneWordCreationRequest,
        SloveneWordCreationResponse,
        SloveneWordInfoResponse,
        SloveneWordMeaning,
        SloveneWordMeaningUpdateRequest,
        SloveneWordMeaningUpdatedResponse,
        SloveneWordMeaningWithDetails,
        SloveneWordMeaningsResponse,
        SloveneWordUpdateRequest,
        SloveneWordWithMeanings,
        SloveneWordsResponse,
        WordErrorReason,
    },
    ids::{CategoryId, SloveneWordId, SloveneWordMeaningId},
};
use reqwest::StatusCode;
use thiserror::Error;

use crate::{
    api::EndpointGroup,
    client::{errors::RequestError, KolomoniHttpClient},
    parsing::{err_if_missing_permissions, unexpected_error_reason, unexpected_response},
    request::{
        typed::{BoundTypedRequest, IntoBoundTypedRequest},
        ToRequestBuilder,
    },
    response::{raw::RawResponse, ResponseValueError},
};




/*
 * Slovene word-related endpoints
 */

pub struct SloveneWordToCreate {
    pub lemma: String,
}



pub struct SloveneWordFieldsToUpdate {
    pub new_lemma: Option<String>,
}

impl SloveneWordFieldsToUpdate {
    pub(crate) fn has_no_fields_to_update(&self) -> bool {
        self.new_lemma.is_none()
    }
}

#[derive(Debug, Error)]
pub enum SloveneWordListFetchingError {
    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for SloveneWordListFetchingError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}



#[derive(Debug, Error)]
pub enum SloveneWordCreationError {
    #[error("a slovene word with this lemma already exists")]
    LemmaAlreadyExists,

    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for SloveneWordCreationError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}


#[derive(Debug, Error)]
pub enum SloveneWordUpdatePreparationError {
    #[error("there were no fields to update")]
    NothingToUpdate,
}

#[derive(Debug, Error)]
pub enum SloveneWordUpdateError {
    #[error("english word not found")]
    NotFound,

    #[error("a slovene word with this lemma already exists")]
    LemmaAlreadyExists,

    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for SloveneWordUpdateError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}


#[derive(Debug, Error)]
pub enum SloveneWordFetchingError {
    #[error("requested slovene word does not exist")]
    NotFound,

    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for SloveneWordFetchingError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}

#[derive(Debug, Error)]
pub enum SloveneWordDeletionError {
    #[error("requested slovene word does not exist")]
    NotFound,

    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for SloveneWordDeletionError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}


fn get_slovene_words_request<'c, C>(
    client: &'c C,
) -> BoundTypedRequest<'c, C, Vec<SloveneWordWithMeanings>, SloveneWordListFetchingError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let response_body = response.into_json_body::<SloveneWordsResponse>().await?;

            Ok(response_body.slovene_words)
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            err_if_missing_permissions::<SloveneWordListFetchingError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .get()
        .endpoint_url("/dictionary/slovene/words")
        .build_request()
        .into_bound_typed_request(response_parser)
}


fn get_slovene_word_by_id_request<'c, C>(
    client: &'c C,
    slovene_word_id: SloveneWordId,
) -> BoundTypedRequest<'c, C, SloveneWordWithMeanings, SloveneWordFetchingError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let response_body = response.into_json_body::<SloveneWordInfoResponse>().await?;

            Ok(response_body.word)
        } else if status == StatusCode::NOT_FOUND {
            let word_error_reason = response.word_error_reason().await?;

            match word_error_reason {
                WordErrorReason::WordNotFound => Err(SloveneWordFetchingError::NotFound),
                _ => Err(unexpected_error_reason(status, word_error_reason)),
            }
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            err_if_missing_permissions::<SloveneWordFetchingError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .get()
        .endpoint_url(format!(
            "/dictionary/slovene/words/{}",
            slovene_word_id
        ))
        .build_request()
        .into_bound_typed_request(response_parser)
}


fn get_slovene_word_by_lemma_request<'c, C>(
    client: &'c C,
    slovene_word_lemma: &str,
) -> BoundTypedRequest<'c, C, SloveneWordWithMeanings, SloveneWordFetchingError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let response_body = response.into_json_body::<SloveneWordInfoResponse>().await?;

            Ok(response_body.word)
        } else if status == StatusCode::NOT_FOUND {
            let word_error_reason = response.word_error_reason().await?;

            match word_error_reason {
                WordErrorReason::WordNotFound => Err(SloveneWordFetchingError::NotFound),
                _ => Err(unexpected_error_reason(status, word_error_reason)),
            }
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            err_if_missing_permissions::<SloveneWordFetchingError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .get()
        .endpoint_url(format!(
            "/dictionary/slovene/words/by-lemma/{}",
            slovene_word_lemma
        ))
        .build_request()
        .into_bound_typed_request(response_parser)
}


fn create_slovene_word_request<'c, C>(
    client: &'c C,
    word_to_create: SloveneWordToCreate,
) -> BoundTypedRequest<'c, C, SloveneWordWithMeanings, SloveneWordCreationError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();


        if status == StatusCode::OK {
            let response_body = response
                .into_json_body::<SloveneWordCreationResponse>()
                .await?;

            Ok(response_body.word)
        } else if status == StatusCode::CONFLICT {
            let word_error_reason = response.word_error_reason().await?;

            match word_error_reason {
                WordErrorReason::WordWithThisLemmaAlreadyExists => {
                    Err(SloveneWordCreationError::LemmaAlreadyExists)
                }
                _ => Err(unexpected_error_reason(status, word_error_reason)),
            }
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            err_if_missing_permissions::<SloveneWordCreationError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .post()
        .endpoint_url("/dictionary/slovene/words")
        .json(&SloveneWordCreationRequest {
            lemma: word_to_create.lemma,
        })
        .build_request()
        .into_bound_typed_request(response_parser)
}


fn update_slovene_word_request<'c, C>(
    client: &'c C,
    slovene_word_id: SloveneWordId,
    fields_to_update: SloveneWordFieldsToUpdate,
) -> Result<
    BoundTypedRequest<'c, C, SloveneWordWithMeanings, SloveneWordUpdateError>,
    SloveneWordUpdatePreparationError,
>
where
    C: KolomoniHttpClient,
{
    if fields_to_update.has_no_fields_to_update() {
        return Err(SloveneWordUpdatePreparationError::NothingToUpdate);
    }


    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let response_body = response.into_json_body::<SloveneWordInfoResponse>().await?;

            Ok(response_body.word)
        } else if status == StatusCode::NOT_FOUND {
            let word_error_reason = response.word_error_reason().await?;

            match word_error_reason {
                WordErrorReason::WordNotFound => Err(SloveneWordUpdateError::NotFound),
                _ => Err(unexpected_error_reason(status, word_error_reason)),
            }
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            err_if_missing_permissions::<SloveneWordUpdateError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else {
            Err(unexpected_response(response).await)
        }
    };

    Ok(client
        .patch()
        .endpoint_url(format!(
            "/dictionary/slovene/words/{}",
            slovene_word_id
        ))
        .json(&SloveneWordUpdateRequest {
            lemma: fields_to_update.new_lemma,
        })
        .build_request()
        .into_bound_typed_request(response_parser))
}


fn delete_slovene_word_request<'c, C>(
    client: &'c C,
    slovene_word_id: SloveneWordId,
) -> BoundTypedRequest<'c, C, (), SloveneWordDeletionError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            Ok(())
        } else if status == StatusCode::NOT_FOUND {
            let word_error_response = response.word_error_reason().await?;

            match word_error_response {
                WordErrorReason::WordNotFound => Err(SloveneWordDeletionError::NotFound),
                _ => Err(unexpected_error_reason(
                    status,
                    word_error_response,
                )),
            }
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            err_if_missing_permissions::<SloveneWordDeletionError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .delete()
        .endpoint_url(format!(
            "/dictionary/slovene/words/{}",
            slovene_word_id
        ))
        .build_request()
        .into_bound_typed_request(response_parser)
}



/*
 * Slovene word meaning-related endpoints
 */


pub struct SloveneWordMeaningToCreate {
    pub disambiguation: Option<String>,

    pub abbreviation: Option<String>,

    pub description: Option<String>,
}

pub struct SloveneWordMeaningFieldsToUpdate {
    pub disambiguation: Option<Option<String>>,

    pub abbreviation: Option<Option<String>>,

    pub description: Option<Option<String>>,
}

impl SloveneWordMeaningFieldsToUpdate {
    pub(crate) fn has_no_fields_to_update(&self) -> bool {
        self.disambiguation.is_none() && self.abbreviation.is_none() && self.description.is_none()
    }
}


#[derive(Debug, Error)]
pub enum SloveneWordMeaningsFetchingError {
    #[error("english word does not exist")]
    WordNotFound,

    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for SloveneWordMeaningsFetchingError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}

#[derive(Debug, Error)]
pub enum SloveneWordMeaningCreationError {
    #[error("slovene word does not exist")]
    WordNotFound,

    #[error("identical slovene word meaning already exists")]
    IdenticalWordMeaningAlreadyExists,

    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for SloveneWordMeaningCreationError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}


#[derive(Debug, Error)]
pub enum SloveneWordMeaningUpdatePreparationError {
    #[error("there were no fields to update")]
    NothingToUpdate,
}


#[derive(Debug, Error)]
pub enum SloveneWordMeaningUpdateError {
    #[error("slovene word does not exist")]
    WordNotFound,

    #[error("slovene word meaning does not exist")]
    WordMeaningNotFound,

    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for SloveneWordMeaningUpdateError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}


#[derive(Debug, Error)]
pub enum SloveneWordMeaningDeletionError {
    #[error("slovene word does not exist")]
    WordNotFound,

    #[error("slovene word meaning does not exist")]
    WordMeaningNotFound,

    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for SloveneWordMeaningDeletionError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}

#[derive(Debug, Error)]
pub enum SloveneWordMeaningCategoryLinkError {
    #[error("slovene word does not exist")]
    WordNotFound,

    #[error("slovene word meaning does not exist")]
    WordMeaningNotFound,

    #[error("slovene word meaning is already linked to the given category")]
    CategoryRelationshipAlreadyExists,

    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for SloveneWordMeaningCategoryLinkError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}

#[derive(Debug, Error)]
pub enum SloveneWordMeaningCategoryUnlinkingError {
    #[error("slovene word does not exist")]
    WordNotFound,

    #[error("slovene word meaning does not exist")]
    WordMeaningNotFound,

    #[error("slovene word meaning-category relationship does not exist")]
    CategoryRelationshipNotFound,

    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for SloveneWordMeaningCategoryUnlinkingError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}




fn get_slovene_word_meanings_request<'c, C>(
    client: &'c C,
    slovene_word_id: SloveneWordId,
) -> BoundTypedRequest<'c, C, Vec<SloveneWordMeaningWithDetails>, SloveneWordMeaningsFetchingError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let response_body = response
                .into_json_body::<SloveneWordMeaningsResponse>()
                .await?;

            Ok(response_body.meanings)
        } else if status == StatusCode::NOT_FOUND {
            let word_error_reason = response.word_error_reason().await?;

            match word_error_reason {
                WordErrorReason::WordNotFound => Err(SloveneWordMeaningsFetchingError::WordNotFound),
                _ => Err(unexpected_error_reason(status, word_error_reason)),
            }
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            err_if_missing_permissions::<SloveneWordMeaningsFetchingError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .get()
        .endpoint_url(format!(
            "/dictionary/slovene/words/{}/meanings",
            slovene_word_id
        ))
        .build_request()
        .into_bound_typed_request(response_parser)
}


fn create_slovene_word_meaning_request<'c, C>(
    client: &'c C,
    slovene_word_id: SloveneWordId,
    word_meaning_to_create: SloveneWordMeaningToCreate,
) -> BoundTypedRequest<'c, C, SloveneWordMeaning, SloveneWordMeaningCreationError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let response_body = response
                .into_json_body::<NewSloveneWordMeaningCreatedResponse>()
                .await?;

            Ok(response_body.meaning)
        } else if status == StatusCode::CONFLICT {
            let word_error_reason = response.word_error_reason().await?;

            match word_error_reason {
                WordErrorReason::IdenticalWordMeaningAlreadyExists => {
                    Err(SloveneWordMeaningCreationError::IdenticalWordMeaningAlreadyExists)
                }
                _ => Err(unexpected_error_reason(status, word_error_reason)),
            }
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            err_if_missing_permissions::<SloveneWordMeaningCreationError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .post()
        .endpoint_url(format!(
            "/dictionary/slovene/words/{}/meanings",
            slovene_word_id
        ))
        .json(&NewSloveneWordMeaningRequest {
            disambiguation: word_meaning_to_create.disambiguation,
            abbreviation: word_meaning_to_create.abbreviation,
            description: word_meaning_to_create.description,
        })
        .build_request()
        .into_bound_typed_request(response_parser)
}


fn update_slovene_word_meaning_request<'c, C>(
    client: &'c C,
    slovene_word_id: SloveneWordId,
    slovene_word_meaning_id: SloveneWordMeaningId,
    fields_to_update: SloveneWordMeaningFieldsToUpdate,
) -> Result<
    BoundTypedRequest<'c, C, SloveneWordMeaningWithDetails, SloveneWordMeaningUpdateError>,
    SloveneWordMeaningUpdatePreparationError,
>
where
    C: KolomoniHttpClient,
{
    if fields_to_update.has_no_fields_to_update() {
        return Err(SloveneWordMeaningUpdatePreparationError::NothingToUpdate);
    }


    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let response_body = response
                .into_json_body::<SloveneWordMeaningUpdatedResponse>()
                .await?;

            Ok(response_body.meaning)
        } else if status == StatusCode::NOT_FOUND {
            let word_error_reason = response.word_error_reason().await?;

            match word_error_reason {
                WordErrorReason::WordNotFound => Err(SloveneWordMeaningUpdateError::WordNotFound),
                WordErrorReason::WordMeaningNotFound => {
                    Err(SloveneWordMeaningUpdateError::WordMeaningNotFound)
                }
                _ => Err(unexpected_error_reason(status, word_error_reason)),
            }
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            err_if_missing_permissions::<SloveneWordMeaningUpdateError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else {
            Err(unexpected_response(response).await)
        }
    };

    Ok(client
        .patch()
        .endpoint_url(format!(
            "/dictionary/slovene/words/{}/meanings/{}",
            slovene_word_id, slovene_word_meaning_id
        ))
        .json(&SloveneWordMeaningUpdateRequest {
            abbreviation: fields_to_update.abbreviation,
            description: fields_to_update.description,
            disambiguation: fields_to_update.disambiguation,
        })
        .build_request()
        .into_bound_typed_request(response_parser))
}


fn delete_slovene_word_meaning_request<'c, C>(
    client: &'c C,
    slovene_word_id: SloveneWordId,
    slovene_word_meaning_id: SloveneWordMeaningId,
) -> BoundTypedRequest<'c, C, (), SloveneWordMeaningDeletionError>
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
                WordErrorReason::WordNotFound => Err(SloveneWordMeaningDeletionError::WordNotFound),
                WordErrorReason::WordMeaningNotFound => {
                    Err(SloveneWordMeaningDeletionError::WordMeaningNotFound)
                }
                _ => Err(unexpected_error_reason(status, word_error_reason)),
            }
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            err_if_missing_permissions::<SloveneWordMeaningDeletionError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .delete()
        .endpoint_url(format!(
            "/dictionary/slovene/words/{}/meanings/{}",
            slovene_word_id, slovene_word_meaning_id
        ))
        .build_request()
        .into_bound_typed_request(response_parser)
}



fn link_category_to_slovene_word_meaning_request<'c, C>(
    client: &'c C,
    slovene_word_id: SloveneWordId,
    slovene_word_meaning_id: SloveneWordMeaningId,
    category_id: CategoryId,
) -> BoundTypedRequest<'c, C, (), SloveneWordMeaningCategoryLinkError>
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
                    Err(SloveneWordMeaningCategoryLinkError::WordNotFound)
                }
                WordErrorReason::WordMeaningNotFound => {
                    Err(SloveneWordMeaningCategoryLinkError::WordMeaningNotFound)
                }
                _ => Err(unexpected_error_reason(status, word_error_reason)),
            }
        } else if status == StatusCode::CONFLICT {
            let word_error_reason = response.word_error_reason().await?;

            match word_error_reason {
                WordErrorReason::WordMeaningAlreadyHasThisCategory => {
                    Err(SloveneWordMeaningCategoryLinkError::CategoryRelationshipAlreadyExists)
                }
                _ => Err(unexpected_error_reason(status, word_error_reason)),
            }
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            err_if_missing_permissions::<SloveneWordMeaningCategoryLinkError>(
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
            "/dictionary/slovene/words/{}/meanings/{}/categories/{}",
            slovene_word_id, slovene_word_meaning_id, category_id
        ))
        .build_request()
        .into_bound_typed_request(response_parser)
}


fn unlink_category_from_slovene_word_meaning_request<'c, C>(
    client: &'c C,
    slovene_word_id: SloveneWordId,
    slovene_word_meaning_id: SloveneWordMeaningId,
    category_id: CategoryId,
) -> BoundTypedRequest<'c, C, (), SloveneWordMeaningCategoryUnlinkingError>
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
                    Err(SloveneWordMeaningCategoryUnlinkingError::WordNotFound)
                }
                WordErrorReason::WordMeaningNotFound => {
                    Err(SloveneWordMeaningCategoryUnlinkingError::WordMeaningNotFound)
                }
                WordErrorReason::WordMeaningCategoryRelationshipNotFound => {
                    Err(SloveneWordMeaningCategoryUnlinkingError::CategoryRelationshipNotFound)
                }
                _ => Err(unexpected_error_reason(status, word_error_reason)),
            }
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            err_if_missing_permissions::<SloveneWordMeaningCategoryUnlinkingError>(
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
            "/dictionary/slovene/words/{}/meanings/{}/categories/{}",
            slovene_word_id, slovene_word_meaning_id, category_id
        ))
        .build_request()
        .into_bound_typed_request(response_parser)
}



pub trait SloveneDictionaryUnauthenticatedEndpoints<'c, C>: EndpointGroup<'c, C>
where
    C: KolomoniHttpClient,
{
    /*
     * Word-related (word meanings are in the next section)
     */
    fn slovene_words(
        &'c self,
    ) -> BoundTypedRequest<'c, C, Vec<SloveneWordWithMeanings>, SloveneWordListFetchingError> {
        get_slovene_words_request(self.client())
    }

    fn slovene_word_by_id(
        &'c self,
        slovene_word_id: SloveneWordId,
    ) -> BoundTypedRequest<'c, C, SloveneWordWithMeanings, SloveneWordFetchingError> {
        get_slovene_word_by_id_request(self.client(), slovene_word_id)
    }

    fn slovene_word_by_lemma(
        &'c self,
        slovene_word_lemma: &str,
    ) -> BoundTypedRequest<'c, C, SloveneWordWithMeanings, SloveneWordFetchingError> {
        get_slovene_word_by_lemma_request(self.client(), slovene_word_lemma)
    }


    /*
     * Word meaning-related (words themselves are in the previous section)
     */
    fn slovene_word_meanings(
        &'c self,
        slovene_word_id: SloveneWordId,
    ) -> BoundTypedRequest<'c, C, Vec<SloveneWordMeaningWithDetails>, SloveneWordMeaningsFetchingError>
    {
        get_slovene_word_meanings_request(self.client(), slovene_word_id)
    }
}


pub trait SloveneDictionaryAuthenticatedEndpoints<'c, C>: EndpointGroup<'c, C>
where
    C: KolomoniHttpClient,
{
    /*
     * Word-related (word meanings are in the next section)
     */
    fn create_slovene_word(
        &'c self,
        word_to_create: SloveneWordToCreate,
    ) -> BoundTypedRequest<'c, C, SloveneWordWithMeanings, SloveneWordCreationError> {
        create_slovene_word_request(self.client(), word_to_create)
    }

    fn update_slovene_word(
        &'c self,
        slovene_word_id: SloveneWordId,
        fields_to_update: SloveneWordFieldsToUpdate,
    ) -> Result<
        BoundTypedRequest<'c, C, SloveneWordWithMeanings, SloveneWordUpdateError>,
        SloveneWordUpdatePreparationError,
    > {
        update_slovene_word_request(self.client(), slovene_word_id, fields_to_update)
    }

    fn delete_slovene_word(
        &'c self,
        slovene_word_id: SloveneWordId,
    ) -> BoundTypedRequest<'c, C, (), SloveneWordDeletionError> {
        delete_slovene_word_request(self.client(), slovene_word_id)
    }


    /*
     * Word meaning-related (words themselves are in the previous section)
     */

    fn create_slovene_word_meaning(
        &'c self,
        slovene_word_id: SloveneWordId,
        word_meaning_to_create: SloveneWordMeaningToCreate,
    ) -> BoundTypedRequest<'c, C, SloveneWordMeaning, SloveneWordMeaningCreationError> {
        create_slovene_word_meaning_request(
            self.client(),
            slovene_word_id,
            word_meaning_to_create,
        )
    }

    fn update_slovene_word_meaning(
        &'c self,
        slovene_word_id: SloveneWordId,
        slovene_word_meaning_id: SloveneWordMeaningId,
        fields_to_update: SloveneWordMeaningFieldsToUpdate,
    ) -> Result<
        BoundTypedRequest<'c, C, SloveneWordMeaningWithDetails, SloveneWordMeaningUpdateError>,
        SloveneWordMeaningUpdatePreparationError,
    > {
        update_slovene_word_meaning_request(
            self.client(),
            slovene_word_id,
            slovene_word_meaning_id,
            fields_to_update,
        )
    }

    fn delete_slovene_word_meaning(
        &'c self,
        slovene_word_id: SloveneWordId,
        slovene_word_meaning_id: SloveneWordMeaningId,
    ) -> BoundTypedRequest<'c, C, (), SloveneWordMeaningDeletionError> {
        delete_slovene_word_meaning_request(
            self.client(),
            slovene_word_id,
            slovene_word_meaning_id,
        )
    }

    fn link_category_to_slovene_word_meaning(
        &'c self,
        slovene_word_id: SloveneWordId,
        slovene_word_meaning_id: SloveneWordMeaningId,
        category_id: CategoryId,
    ) -> BoundTypedRequest<'c, C, (), SloveneWordMeaningCategoryLinkError> {
        link_category_to_slovene_word_meaning_request(
            self.client(),
            slovene_word_id,
            slovene_word_meaning_id,
            category_id,
        )
    }

    fn unlink_category_from_slovene_word_meaning(
        &'c self,
        slovene_word_id: SloveneWordId,
        slovene_word_meaning_id: SloveneWordMeaningId,
        category_id: CategoryId,
    ) -> BoundTypedRequest<'c, C, (), SloveneWordMeaningCategoryUnlinkingError> {
        unlink_category_from_slovene_word_meaning_request(
            self.client(),
            slovene_word_id,
            slovene_word_meaning_id,
            category_id,
        )
    }
}


pub struct SloveneDictionaryUnauthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
    client: &'c C,
}

impl<'c, C> SloveneDictionaryUnauthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
    pub(crate) const fn new(client: &'c C) -> Self {
        Self { client }
    }
}

impl<'c, C> EndpointGroup<'c, C> for SloveneDictionaryUnauthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
    fn client(&'c self) -> &'c C {
        self.client
    }
}

impl<'c, C> SloveneDictionaryUnauthenticatedEndpoints<'c, C>
    for SloveneDictionaryUnauthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
}



pub struct SloveneDictionaryAuthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
    client: &'c C,
}

impl<'c, C> SloveneDictionaryAuthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
    pub(crate) const fn new(client: &'c C) -> Self {
        Self { client }
    }
}


impl<'c, C> EndpointGroup<'c, C> for SloveneDictionaryAuthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
    fn client(&'c self) -> &'c C {
        self.client
    }
}

impl<'c, C> SloveneDictionaryUnauthenticatedEndpoints<'c, C>
    for SloveneDictionaryAuthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
}


impl<'c, C> SloveneDictionaryAuthenticatedEndpoints<'c, C>
    for SloveneDictionaryAuthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
}
