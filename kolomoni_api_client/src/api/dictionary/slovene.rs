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
        SloveneWordMeaningWithCategoriesAndTranslations,
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
    errors::{ClientError, ClientResult},
    macros::{
        handle_error_reasons_or_catch_unexpected_status,
        handle_uncaught_status_code,
        handle_unexpected_error_reason,
        handlers,
    },
    request::RequestBuilder,
    ApiClient,
    AuthenticatedApiClient,
};
use crate::{request::ApiClientRequestBuild, UnauthenticatedApiClient};




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
pub enum SloveneWordCreationError {
    #[error("a slovene word with this lemma already exists")]
    LemmaAlreadyExists,

    #[error(transparent)]
    ClientError {
        #[from]
        error: ClientError,
    },
}

#[derive(Debug, Error)]
pub enum SloveneWordUpdatingError {
    #[error("english word not found")]
    NotFound,

    #[error("a slovene word with this lemma already exists")]
    LemmaAlreadyExists,

    #[error("there were no fields to update")]
    NoFieldsToUpdate,

    #[error(transparent)]
    ClientError {
        #[from]
        error: ClientError,
    },
}

#[derive(Debug, Error)]
pub enum SloveneWordFetchingError {
    #[error("requested slovene word does not exist")]
    NotFound,

    #[error(transparent)]
    ClientError {
        #[from]
        error: ClientError,
    },
}

#[derive(Debug, Error)]
pub enum SloveneWordDeletionError {
    #[error("requested slovene word does not exist")]
    NotFound,

    #[error(transparent)]
    ClientError {
        #[from]
        error: ClientError,
    },
}


async fn get_slovene_words<C>(client: &C) -> ClientResult<Vec<SloveneWordWithMeanings>>
where
    C: ApiClient,
{
    let response = client
        .get_request_builder()
        .endpoint_url("/dictionary/slovene/words")
        .send()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        let response_body = response.json::<SloveneWordsResponse>().await?;

        Ok(response_body.slovene_words)
    } else if response_status == StatusCode::FORBIDDEN {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::MissingPermissions]);
    } else {
        handle_uncaught_status_code!(response_status);
    }
}


async fn get_slovene_word_by_id<C>(
    client: &C,
    slovene_word_id: SloveneWordId,
) -> ClientResult<SloveneWordWithMeanings, SloveneWordFetchingError>
where
    C: ApiClient,
{
    let response = client
        .get_request_builder()
        .endpoint_url(format!(
            "/dictionary/slovene/words/{}",
            slovene_word_id
        ))
        .send()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        let response_body = response.json::<SloveneWordInfoResponse>().await?;

        Ok(response_body.word)
    } else if response_status == StatusCode::NOT_FOUND {
        let word_error_reason = response.word_error_reason().await?;

        match word_error_reason {
            WordErrorReason::WordNotFound => Err(SloveneWordFetchingError::NotFound),
            _ => handle_unexpected_error_reason!(word_error_reason, response_status),
        }
    } else if response_status == StatusCode::FORBIDDEN {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::MissingPermissions]);
    } else {
        handle_uncaught_status_code!(response_status);
    }
}


async fn get_slovene_word_by_lemma<C>(
    client: &C,
    slovene_word_lemma: &str,
) -> ClientResult<SloveneWordWithMeanings, SloveneWordFetchingError>
where
    C: ApiClient,
{
    let response = client
        .get_request_builder()
        .endpoint_url(format!(
            "/dictionary/slovene/words/by-lemma/{}",
            slovene_word_lemma
        ))
        .send()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        let response_body = response.json::<SloveneWordInfoResponse>().await?;

        Ok(response_body.word)
    } else if response_status == StatusCode::NOT_FOUND {
        let word_error_reason = response.word_error_reason().await?;

        match word_error_reason {
            WordErrorReason::WordNotFound => Err(SloveneWordFetchingError::NotFound),
            _ => handle_unexpected_error_reason!(word_error_reason, response_status),
        }
    } else if response_status == StatusCode::FORBIDDEN {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::MissingPermissions]);
    } else {
        handle_uncaught_status_code!(response_status);
    }
}


async fn create_slovene_word<C>(
    client: &C,
    word_to_create: SloveneWordToCreate,
) -> ClientResult<SloveneWordWithMeanings, SloveneWordCreationError>
where
    C: AuthenticatedApiClient,
{
    let response = client
        .post_request_builder()
        .endpoint_url("/dictionary/slovene/words")
        .json(&SloveneWordCreationRequest {
            lemma: word_to_create.lemma,
        })
        .send()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        let response_body = response.json::<SloveneWordCreationResponse>().await?;

        Ok(response_body.word)
    } else if response_status == StatusCode::CONFLICT {
        let word_error_reason = response.word_error_reason().await?;

        match word_error_reason {
            WordErrorReason::WordWithThisLemmaAlreadyExists => {
                Err(SloveneWordCreationError::LemmaAlreadyExists)
            }
            _ => handle_unexpected_error_reason!(word_error_reason, response_status),
        }
    } else if response_status == StatusCode::FORBIDDEN {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::MissingPermissions]);
    } else {
        handle_uncaught_status_code!(response_status);
    }
}


async fn update_slovene_word<C>(
    client: &C,
    slovene_word_id: SloveneWordId,
    fields_to_update: SloveneWordFieldsToUpdate,
) -> ClientResult<SloveneWordWithMeanings, SloveneWordUpdatingError>
where
    C: AuthenticatedApiClient,
{
    if fields_to_update.has_no_fields_to_update() {
        return Err(SloveneWordUpdatingError::NoFieldsToUpdate);
    }


    let response = client
        .patch_request_builder()
        .endpoint_url(format!(
            "/dictionary/slovene/words/{}",
            slovene_word_id
        ))
        .json(&SloveneWordUpdateRequest {
            lemma: fields_to_update.new_lemma,
        })
        .send()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        let response_body = response.json::<SloveneWordInfoResponse>().await?;

        Ok(response_body.word)
    } else if response_status == StatusCode::NOT_FOUND {
        let word_error_reason = response.word_error_reason().await?;

        match word_error_reason {
            WordErrorReason::WordNotFound => Err(SloveneWordUpdatingError::NotFound),
            _ => handle_unexpected_error_reason!(word_error_reason, response_status),
        }
    } else if response_status == StatusCode::FORBIDDEN {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::MissingPermissions]);
    } else {
        handle_uncaught_status_code!(response_status);
    }
}


async fn delete_slovene_word<C>(
    client: &C,
    slovene_word_id: SloveneWordId,
) -> ClientResult<(), SloveneWordDeletionError>
where
    C: AuthenticatedApiClient,
{
    let response = client
        .delete_request_builder()
        .endpoint_url(format!(
            "/dictionary/slovene/words/{}",
            slovene_word_id
        ))
        .send()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        Ok(())
    } else if response_status == StatusCode::NOT_FOUND {
        let word_error_response = response.word_error_reason().await?;

        match word_error_response {
            WordErrorReason::WordNotFound => Err(SloveneWordDeletionError::NotFound),
            _ => handle_unexpected_error_reason!(word_error_response, response_status),
        }
    } else if response_status == StatusCode::FORBIDDEN {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::MissingPermissions]);
    } else {
        handle_uncaught_status_code!(response_status);
    }
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
    ClientError {
        #[from]
        error: ClientError,
    },
}

#[derive(Debug, Error)]
pub enum SloveneWordMeaningCreationError {
    #[error("slovene word does not exist")]
    WordNotFound,

    #[error("identical slovene word meaning already exists")]
    IdenticalWordMeaningAlreadyExists,

    #[error(transparent)]
    ClientError {
        #[from]
        error: ClientError,
    },
}



#[derive(Debug, Error)]
pub enum SloveneWordMeaningUpdatingError {
    #[error("slovene word does not exist")]
    WordNotFound,

    #[error("slovene word meaning does not exist")]
    WordMeaningNotFound,

    #[error("there were no fields to update")]
    NoFieldsToUpdate,

    #[error(transparent)]
    ClientError {
        #[from]
        error: ClientError,
    },
}

#[derive(Debug, Error)]
pub enum SloveneWordMeaningDeletionError {
    #[error("slovene word does not exist")]
    WordNotFound,

    #[error("slovene word meaning does not exist")]
    WordMeaningNotFound,

    #[error(transparent)]
    ClientError {
        #[from]
        error: ClientError,
    },
}

#[derive(Debug, Error)]
pub enum SloveneWordMeaningCategoryLinkingError {
    #[error("slovene word does not exist")]
    WordNotFound,

    #[error("slovene word meaning does not exist")]
    WordMeaningNotFound,

    #[error("slovene word meaning is already linked to the given category")]
    CategoryRelationshipAlreadyExists,

    #[error(transparent)]
    ClientError {
        #[from]
        error: ClientError,
    },
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
    ClientError {
        #[from]
        error: ClientError,
    },
}




async fn get_slovene_word_meanings<C>(
    client: &C,
    slovene_word_id: SloveneWordId,
) -> ClientResult<
    Vec<SloveneWordMeaningWithCategoriesAndTranslations>,
    SloveneWordMeaningsFetchingError,
>
where
    C: ApiClient,
{
    let response = RequestBuilder::get(client)
        .endpoint_url(format!(
            "/dictionary/slovene/words/{}/meanings",
            slovene_word_id
        ))
        .send()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        let response_body = response.json::<SloveneWordMeaningsResponse>().await?;

        Ok(response_body.meanings)
    } else if response_status == StatusCode::NOT_FOUND {
        let word_error_reason = response.word_error_reason().await?;

        match word_error_reason {
            WordErrorReason::WordNotFound => Err(SloveneWordMeaningsFetchingError::WordNotFound),
            _ => handle_unexpected_error_reason!(word_error_reason, response_status),
        }
    } else if response_status == StatusCode::FORBIDDEN {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::MissingPermissions]);
    } else {
        handle_uncaught_status_code!(response_status);
    }
}


async fn create_slovene_word_meaning<C>(
    client: &C,
    slovene_word_id: SloveneWordId,
    word_meaning_to_create: SloveneWordMeaningToCreate,
) -> ClientResult<SloveneWordMeaning, SloveneWordMeaningCreationError>
where
    C: AuthenticatedApiClient,
{
    let response = client
        .post_request_builder()
        .endpoint_url(format!(
            "/dictionary/slovene/words/{}/meanings",
            slovene_word_id
        ))
        .json(&NewSloveneWordMeaningRequest {
            disambiguation: word_meaning_to_create.disambiguation,
            abbreviation: word_meaning_to_create.abbreviation,
            description: word_meaning_to_create.description,
        })
        .send()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        let response_body = response
            .json::<NewSloveneWordMeaningCreatedResponse>()
            .await?;

        Ok(response_body.meaning)
    } else if response_status == StatusCode::CONFLICT {
        let word_error_reason = response.word_error_reason().await?;

        match word_error_reason {
            WordErrorReason::IdenticalWordMeaningAlreadyExists => {
                Err(SloveneWordMeaningCreationError::IdenticalWordMeaningAlreadyExists)
            }
            _ => handle_unexpected_error_reason!(word_error_reason, response_status),
        }
    } else if response_status == StatusCode::FORBIDDEN {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::MissingPermissions]);
    } else {
        handle_uncaught_status_code!(response_status);
    }
}


async fn update_slovene_word_meaning<C>(
    client: &C,
    slovene_word_id: SloveneWordId,
    slovene_word_meaning_id: SloveneWordMeaningId,
    fields_to_update: SloveneWordMeaningFieldsToUpdate,
) -> ClientResult<SloveneWordMeaningWithCategoriesAndTranslations, SloveneWordMeaningUpdatingError>
where
    C: AuthenticatedApiClient,
{
    if fields_to_update.has_no_fields_to_update() {
        return Err(SloveneWordMeaningUpdatingError::NoFieldsToUpdate);
    }


    let response = client
        .patch_request_builder()
        .endpoint_url(format!(
            "/dictionary/slovene/words/{}/meanings/{}",
            slovene_word_id, slovene_word_meaning_id
        ))
        .json(&SloveneWordMeaningUpdateRequest {
            abbreviation: fields_to_update.abbreviation,
            description: fields_to_update.description,
            disambiguation: fields_to_update.disambiguation,
        })
        .send()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        let response_body = response.json::<SloveneWordMeaningUpdatedResponse>().await?;

        Ok(response_body.meaning)
    } else if response_status == StatusCode::NOT_FOUND {
        let word_error_reason = response.word_error_reason().await?;

        match word_error_reason {
            WordErrorReason::WordNotFound => Err(SloveneWordMeaningUpdatingError::WordNotFound),
            WordErrorReason::WordMeaningNotFound => {
                Err(SloveneWordMeaningUpdatingError::WordMeaningNotFound)
            }
            _ => handle_unexpected_error_reason!(word_error_reason, response_status),
        }
    } else if response_status == StatusCode::FORBIDDEN {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::MissingPermissions]);
    } else {
        handle_uncaught_status_code!(response_status);
    }
}


async fn delete_slovene_word_meaning<C>(
    client: &C,
    slovene_word_id: SloveneWordId,
    slovene_word_meaning_id: SloveneWordMeaningId,
) -> ClientResult<(), SloveneWordMeaningDeletionError>
where
    C: AuthenticatedApiClient,
{
    let response = client
        .delete_request_builder()
        .endpoint_url(format!(
            "/dictionary/slovene/words/{}/meanings/{}",
            slovene_word_id, slovene_word_meaning_id
        ))
        .send()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        Ok(())
    } else if response_status == StatusCode::NOT_FOUND {
        let word_error_reason = response.word_error_reason().await?;

        match word_error_reason {
            WordErrorReason::WordNotFound => Err(SloveneWordMeaningDeletionError::WordNotFound),
            WordErrorReason::WordMeaningNotFound => {
                Err(SloveneWordMeaningDeletionError::WordMeaningNotFound)
            }
            _ => handle_unexpected_error_reason!(word_error_reason, response_status),
        }
    } else if response_status == StatusCode::FORBIDDEN {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::MissingPermissions]);
    } else {
        handle_uncaught_status_code!(response_status);
    }
}



async fn link_category_to_slovene_word_meaning<C>(
    client: &C,
    slovene_word_id: SloveneWordId,
    slovene_word_meaning_id: SloveneWordMeaningId,
    category_id: CategoryId,
) -> ClientResult<(), SloveneWordMeaningCategoryLinkingError>
where
    C: AuthenticatedApiClient,
{
    let response = client
        .post_request_builder()
        .endpoint_url(format!(
            "/dictionary/slovene/words/{}/meanings/{}/categories/{}",
            slovene_word_id, slovene_word_meaning_id, category_id
        ))
        .send()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        Ok(())
    } else if response_status == StatusCode::NOT_FOUND {
        let word_error_reason = response.word_error_reason().await?;

        match word_error_reason {
            WordErrorReason::WordNotFound => {
                Err(SloveneWordMeaningCategoryLinkingError::WordNotFound)
            }
            WordErrorReason::WordMeaningNotFound => {
                Err(SloveneWordMeaningCategoryLinkingError::WordMeaningNotFound)
            }
            _ => handle_unexpected_error_reason!(word_error_reason, response_status),
        }
    } else if response_status == StatusCode::CONFLICT {
        let word_error_reason = response.word_error_reason().await?;

        match word_error_reason {
            WordErrorReason::WordMeaningAlreadyHasThisCategory => {
                Err(SloveneWordMeaningCategoryLinkingError::CategoryRelationshipAlreadyExists)
            }
            _ => handle_unexpected_error_reason!(word_error_reason, response_status),
        }
    } else if response_status == StatusCode::FORBIDDEN {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::MissingPermissions]);
    } else {
        handle_uncaught_status_code!(response_status);
    }
}


async fn unlink_category_from_slovene_word_meaning<C>(
    client: &C,
    slovene_word_id: SloveneWordId,
    slovene_word_meaning_id: SloveneWordMeaningId,
    category_id: CategoryId,
) -> ClientResult<(), SloveneWordMeaningCategoryUnlinkingError>
where
    C: AuthenticatedApiClient,
{
    let response = client
        .delete_request_builder()
        .endpoint_url(format!(
            "/dictionary/slovene/words/{}/meanings/{}/categories/{}",
            slovene_word_id, slovene_word_meaning_id, category_id
        ))
        .send()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        Ok(())
    } else if response_status == StatusCode::NOT_FOUND {
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
            _ => handle_unexpected_error_reason!(word_error_reason, response_status),
        }
    } else if response_status == StatusCode::FORBIDDEN {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::MissingPermissions]);
    } else {
        handle_uncaught_status_code!(response_status);
    }
}




pub struct SloveneDictionaryUnauthenticatedApi<'c, C>
where
    C: UnauthenticatedApiClient,
{
    client: &'c C,
}

impl<'c, C> SloveneDictionaryUnauthenticatedApi<'c, C>
where
    C: UnauthenticatedApiClient,
{
    pub(crate) const fn new(client: &'c C) -> Self {
        Self { client }
    }

    /*
     * Word-related (word meanings are in the next section)
     */
    pub async fn slovene_words(&self) -> ClientResult<Vec<SloveneWordWithMeanings>> {
        get_slovene_words(self.client).await
    }

    pub async fn slovene_word_by_id(
        &self,
        slovene_word_id: SloveneWordId,
    ) -> ClientResult<SloveneWordWithMeanings, SloveneWordFetchingError> {
        get_slovene_word_by_id(self.client, slovene_word_id).await
    }

    pub async fn slovene_word_by_lemma(
        &self,
        slovene_word_lemma: &str,
    ) -> ClientResult<SloveneWordWithMeanings, SloveneWordFetchingError> {
        get_slovene_word_by_lemma(self.client, slovene_word_lemma).await
    }


    /*
     * Word meaning-related (words themselves are in the previous section)
     */
    pub async fn slovene_word_meanings(
        &self,
        slovene_word_id: SloveneWordId,
    ) -> ClientResult<
        Vec<SloveneWordMeaningWithCategoriesAndTranslations>,
        SloveneWordMeaningsFetchingError,
    > {
        get_slovene_word_meanings(self.client, slovene_word_id).await
    }
}



pub struct SloveneDictionaryAuthenticatedApi<'c, C>
where
    C: AuthenticatedApiClient,
{
    client: &'c C,
}

impl<'c, C> SloveneDictionaryAuthenticatedApi<'c, C>
where
    C: AuthenticatedApiClient,
{
    pub(crate) const fn new(client: &'c C) -> Self {
        Self { client }
    }

    /*
     * Word-related (word meanings are in the next section)
     */
    pub async fn slovene_words(&self) -> ClientResult<Vec<SloveneWordWithMeanings>> {
        get_slovene_words(self.client).await
    }

    pub async fn slovene_word_by_id(
        &self,
        slovene_word_id: SloveneWordId,
    ) -> ClientResult<SloveneWordWithMeanings, SloveneWordFetchingError> {
        get_slovene_word_by_id(self.client, slovene_word_id).await
    }

    pub async fn slovene_word_by_lemma(
        &self,
        slovene_word_lemma: &str,
    ) -> ClientResult<SloveneWordWithMeanings, SloveneWordFetchingError> {
        get_slovene_word_by_lemma(self.client, slovene_word_lemma).await
    }

    pub async fn create_slovene_word(
        &self,
        word_to_create: SloveneWordToCreate,
    ) -> ClientResult<SloveneWordWithMeanings, SloveneWordCreationError> {
        create_slovene_word(self.client, word_to_create).await
    }

    pub async fn update_slovene_word(
        &self,
        slovene_word_id: SloveneWordId,
        fields_to_update: SloveneWordFieldsToUpdate,
    ) -> ClientResult<SloveneWordWithMeanings, SloveneWordUpdatingError> {
        update_slovene_word(self.client, slovene_word_id, fields_to_update).await
    }

    pub async fn delete_slovene_word(
        &self,
        slovene_word_id: SloveneWordId,
    ) -> ClientResult<(), SloveneWordDeletionError> {
        delete_slovene_word(self.client, slovene_word_id).await
    }


    /*
     * Word meaning-related (words themselves are in the previous section)
     */
    pub async fn slovene_word_meanings(
        &self,
        slovene_word_id: SloveneWordId,
    ) -> ClientResult<
        Vec<SloveneWordMeaningWithCategoriesAndTranslations>,
        SloveneWordMeaningsFetchingError,
    > {
        get_slovene_word_meanings(self.client, slovene_word_id).await
    }

    pub async fn create_slovene_word_meaning(
        &self,
        slovene_word_id: SloveneWordId,
        word_meaning_to_create: SloveneWordMeaningToCreate,
    ) -> ClientResult<SloveneWordMeaning, SloveneWordMeaningCreationError> {
        create_slovene_word_meaning(
            self.client,
            slovene_word_id,
            word_meaning_to_create,
        )
        .await
    }

    pub async fn update_slovene_word_meaning(
        &self,
        slovene_word_id: SloveneWordId,
        slovene_word_meaning_id: SloveneWordMeaningId,
        fields_to_update: SloveneWordMeaningFieldsToUpdate,
    ) -> ClientResult<SloveneWordMeaningWithCategoriesAndTranslations, SloveneWordMeaningUpdatingError>
    {
        update_slovene_word_meaning(
            self.client,
            slovene_word_id,
            slovene_word_meaning_id,
            fields_to_update,
        )
        .await
    }

    pub async fn delete_slovene_word_meaning(
        &self,
        slovene_word_id: SloveneWordId,
        slovene_word_meaning_id: SloveneWordMeaningId,
    ) -> ClientResult<(), SloveneWordMeaningDeletionError> {
        delete_slovene_word_meaning(
            self.client,
            slovene_word_id,
            slovene_word_meaning_id,
        )
        .await
    }

    pub async fn link_category_to_slovene_word_meaning(
        &self,
        slovene_word_id: SloveneWordId,
        slovene_word_meaning_id: SloveneWordMeaningId,
        category_id: CategoryId,
    ) -> ClientResult<(), SloveneWordMeaningCategoryLinkingError> {
        link_category_to_slovene_word_meaning(
            self.client,
            slovene_word_id,
            slovene_word_meaning_id,
            category_id,
        )
        .await
    }

    pub async fn unlink_category_from_slovene_word_meaning(
        &self,
        slovene_word_id: SloveneWordId,
        slovene_word_meaning_id: SloveneWordMeaningId,
        category_id: CategoryId,
    ) -> ClientResult<(), SloveneWordMeaningCategoryUnlinkingError> {
        unlink_category_from_slovene_word_meaning(
            self.client,
            slovene_word_id,
            slovene_word_meaning_id,
            category_id,
        )
        .await
    }
}
