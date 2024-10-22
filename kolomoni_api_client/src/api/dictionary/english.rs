use chrono::{DateTime, Utc};
use kolomoni_core::{
    api_models::{
        EnglishWordCreationRequest,
        EnglishWordCreationResponse,
        EnglishWordInfoResponse,
        EnglishWordMeaning,
        EnglishWordMeaningUpdateRequest,
        EnglishWordMeaningUpdatedResponse,
        EnglishWordMeaningWithCategoriesAndTranslations,
        EnglishWordMeaningsResponse,
        EnglishWordUpdateRequest,
        EnglishWordWithMeanings,
        EnglishWordsResponse,
        NewEnglishWordMeaningCreatedResponse,
        NewEnglishWordMeaningRequest,
        WordErrorReason,
    },
    ids::{EnglishWordId, EnglishWordMeaningId},
};
use reqwest::StatusCode;
use thiserror::Error;

use crate::{
    errors::{ClientError, ClientResult},
    macros::{
        handle_error_reasons_or_catch_unexpected_status,
        handle_internal_server_error,
        handle_unexpected_status_code,
        handlers,
        unexpected_error_reason,
    },
    request::RequestBuilder,
    AuthenticatedClient,
    Client,
    HttpClient,
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
pub enum EnglishWordFetchingError {
    #[error("english word not found")]
    NotFound,

    #[error(transparent)]
    ClientError {
        #[from]
        error: ClientError,
    },
}

#[derive(Debug, Error)]
pub enum EnglishWordCreationError {
    #[error("an english word with this lemma already exists")]
    LemmaAlreadyExists,

    #[error(transparent)]
    ClientError {
        #[from]
        error: ClientError,
    },
}


#[derive(Debug, Error)]
pub enum EnglishWordUpdatingError {
    #[error("english word not found")]
    NotFound,

    #[error("english word with this lemma already exists")]
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
pub enum EnglishWordDeletionError {
    #[error("english word not found")]
    NotFound,

    #[error(transparent)]
    ClientError {
        #[from]
        error: ClientError,
    },
}


/*
 * English word-related endpoints
 */


async fn get_english_words<C>(
    client: &C,
    options: EnglishWordFetchingOptions,
) -> ClientResult<Vec<EnglishWordWithMeanings>>
where
    C: HttpClient,
{
    let request = if let Some(only_last_modified_after) = options.only_words_modified_after {
        RequestBuilder::get(client).endpoint_url_with_parameters(
            "/dictionary/english",
            [(
                "last_modified_after",
                only_last_modified_after.to_rfc3339(),
            )],
        )
    } else {
        RequestBuilder::get(client).endpoint_url("/dictionary/english")
    };

    let response = request.send().await?;
    let response_status = response.status();


    if response_status == StatusCode::OK {
        let response_body = response.json::<EnglishWordsResponse>().await?;

        Ok(response_body.english_words)
    } else if response_status == StatusCode::FORBIDDEN {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::MissingPermissions]);
    } else {
        handle_unexpected_status_code!(response_status);
    }
}


async fn get_english_word_by_id<C>(
    client: &C,
    english_word_id: EnglishWordId,
) -> ClientResult<EnglishWordWithMeanings, EnglishWordFetchingError>
where
    C: HttpClient,
{
    let response = RequestBuilder::get(client)
        .endpoint_url(format!(
            "/dictionary/english/{}",
            english_word_id.into_uuid()
        ))
        .send()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        let response_body = response.json::<EnglishWordInfoResponse>().await?;

        Ok(response_body.word)
    } else if response_status == StatusCode::NOT_FOUND {
        let error_reason = response.word_error_reason().await?;

        match error_reason {
            WordErrorReason::WordNotFound => Err(EnglishWordFetchingError::NotFound),
            _ => unexpected_error_reason!(error_reason, response_status),
        }
    } else if response_status == StatusCode::BAD_REQUEST {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::InvalidUuidFormat]);
    } else {
        handle_unexpected_status_code!(response_status);
    }
}


async fn get_english_word_by_lemma<C>(
    client: &C,
    lemma: &str,
) -> ClientResult<EnglishWordWithMeanings, EnglishWordFetchingError>
where
    C: HttpClient,
{
    let response = RequestBuilder::get(client)
        .endpoint_url(format!("/dictionary/english/by-lemma/{}", lemma))
        .send()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        let response_body = response.json::<EnglishWordInfoResponse>().await?;

        Ok(response_body.word)
    } else if response_status == StatusCode::NOT_FOUND {
        let word_error_reason = response.word_error_reason().await?;

        match word_error_reason {
            WordErrorReason::WordNotFound => Err(EnglishWordFetchingError::NotFound),
            _ => unexpected_error_reason!(word_error_reason, response_status),
        }
    } else if response_status == StatusCode::BAD_REQUEST {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::InvalidUuidFormat]);
    } else if response_status == StatusCode::FORBIDDEN {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::MissingPermissions]);
    } else {
        handle_unexpected_status_code!(response_status);
    }
}


async fn create_english_word(
    client: &AuthenticatedClient,
    word: EnglishWordToCreate,
) -> ClientResult<EnglishWordWithMeanings, EnglishWordCreationError> {
    let response = RequestBuilder::post(client)
        .endpoint_url("/dictionary/english")
        .json(&EnglishWordCreationRequest { lemma: word.lemma })
        .send()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        let response_body = response.json::<EnglishWordCreationResponse>().await?;

        Ok(response_body.word)
    } else if response_status == StatusCode::CONFLICT {
        let word_error_reason = response.word_error_reason().await?;

        match word_error_reason {
            WordErrorReason::WordWithGivenLemmaAlreadyExists => {
                Err(EnglishWordCreationError::LemmaAlreadyExists)
            }
            _ => unexpected_error_reason!(word_error_reason, response_status),
        }
    } else if response_status == StatusCode::FORBIDDEN {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::MissingPermissions]);
    } else {
        handle_unexpected_status_code!(response_status);
    }
}


async fn update_english_word(
    client: &AuthenticatedClient,
    english_word_id: EnglishWordId,
    fields_to_update: EnglishWordFieldsToUpdate,
) -> ClientResult<EnglishWordWithMeanings, EnglishWordUpdatingError> {
    if fields_to_update.has_no_fields_to_update() {
        return Err(EnglishWordUpdatingError::NoFieldsToUpdate);
    }


    let response = RequestBuilder::patch(client)
        .endpoint_url(format!("/dictionary/english/{}", english_word_id))
        .json(&EnglishWordUpdateRequest {
            lemma: fields_to_update.new_lemma,
        })
        .send()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        let response_body = response.json::<EnglishWordInfoResponse>().await?;

        Ok(response_body.word)
    } else if response_status == StatusCode::NOT_FOUND {
        let word_error_reason = response.word_error_reason().await?;

        match word_error_reason {
            WordErrorReason::WordNotFound => Err(EnglishWordUpdatingError::NotFound),
            _ => unexpected_error_reason!(word_error_reason, response_status),
        }
    } else if response_status == StatusCode::CONFLICT {
        let word_error_reason = response.word_error_reason().await?;

        match word_error_reason {
            WordErrorReason::WordWithGivenLemmaAlreadyExists => {
                Err(EnglishWordUpdatingError::LemmaAlreadyExists)
            }
            _ => unexpected_error_reason!(word_error_reason, response_status),
        }
    } else {
        handle_unexpected_status_code!(response_status);
    }
}

async fn delete_english_word(
    client: &AuthenticatedClient,
    english_word_id: EnglishWordId,
) -> ClientResult<(), EnglishWordDeletionError> {
    let response = RequestBuilder::delete(client)
        .endpoint_url(format!("/dictionary/english/{}", english_word_id))
        .send()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        Ok(())
    } else if response_status == StatusCode::NOT_FOUND {
        let word_error_reason = response.word_error_reason().await?;

        match word_error_reason {
            WordErrorReason::WordNotFound => Err(EnglishWordDeletionError::NotFound),
            _ => unexpected_error_reason!(word_error_reason, response_status),
        }
    } else if response_status == StatusCode::BAD_REQUEST {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::InvalidUuidFormat]);
    } else if response_status == StatusCode::FORBIDDEN {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::MissingPermissions]);
    } else {
        handle_unexpected_status_code!(response_status);
    }
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
    ClientError {
        #[from]
        error: ClientError,
    },
}


#[derive(Debug, Error)]
pub enum EnglishWordMeaningCreationError {
    #[error("english word does not exist")]
    WordNotFound,

    #[error("identical english word meaning already exists")]
    IdenticalWordMeaningAlreadyExists,

    #[error(transparent)]
    ClientError {
        #[from]
        error: ClientError,
    },
}


#[derive(Debug, Error)]
pub enum EnglishWordMeaningUpdatingError {
    #[error("english word does not exist")]
    WordNotFound,

    #[error("english word meaning does not exist")]
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
pub enum EnglishWordMeaningDeletionError {
    #[error("english word does not exist")]
    WordNotFound,

    #[error("english word meaning does not exist")]
    WordMeaningNotFound,

    #[error(transparent)]
    ClientError {
        #[from]
        error: ClientError,
    },
}


async fn get_english_word_meanings<C>(
    client: &C,
    english_word_id: EnglishWordId,
) -> ClientResult<
    Vec<EnglishWordMeaningWithCategoriesAndTranslations>,
    EnglishWordMeaningsFetchingError,
>
where
    C: HttpClient,
{
    let response = RequestBuilder::get(client)
        .endpoint_url(format!(
            "/dictionary/english/{}/meaning",
            english_word_id
        ))
        .send()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        let response_body = response.json::<EnglishWordMeaningsResponse>().await?;

        Ok(response_body.meanings)
    } else if response_status == StatusCode::NOT_FOUND {
        let word_error_response = response.word_error_reason().await?;

        match word_error_response {
            WordErrorReason::WordNotFound => Err(EnglishWordMeaningsFetchingError::WordNotFound),
            _ => unexpected_error_reason!(word_error_response, response_status),
        }
    } else if response_status == StatusCode::FORBIDDEN {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::MissingPermissions]);
    } else {
        handle_unexpected_status_code!(response_status);
    }
}


async fn create_english_word_meaning(
    client: &AuthenticatedClient,
    english_word_id: EnglishWordId,
    word_meaning_to_create: EnglishWordMeaningToCreate,
) -> ClientResult<EnglishWordMeaning, EnglishWordMeaningCreationError> {
    let response = RequestBuilder::post(client)
        .endpoint_url(format!(
            "/dictionary/english/{}/meaning",
            english_word_id
        ))
        .json(&NewEnglishWordMeaningRequest {
            abbreviation: word_meaning_to_create.abbreviation,
            disambiguation: word_meaning_to_create.disambiguation,
            description: word_meaning_to_create.description,
        })
        .send()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        let response_body = response
            .json::<NewEnglishWordMeaningCreatedResponse>()
            .await?;

        Ok(response_body.meaning)
    } else if response_status == StatusCode::CONFLICT {
        let word_error_reason = response.word_error_reason().await?;

        match word_error_reason {
            WordErrorReason::IdenticalWordMeaningAlreadyExists => {
                Err(EnglishWordMeaningCreationError::IdenticalWordMeaningAlreadyExists)
            }
            _ => unexpected_error_reason!(word_error_reason, response_status),
        }
    } else if response_status == StatusCode::FORBIDDEN {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::MissingPermissions]);
    } else {
        handle_unexpected_status_code!(response_status);
    }
}


async fn update_english_word_meaning(
    client: &AuthenticatedClient,
    english_word_id: EnglishWordId,
    english_word_meaning_id: EnglishWordMeaningId,
    fields_to_update: EnglishWordMeaningFieldsToUpdate,
) -> ClientResult<EnglishWordMeaningWithCategoriesAndTranslations, EnglishWordMeaningUpdatingError> {
    if fields_to_update.has_no_fields_to_update() {
        return Err(EnglishWordMeaningUpdatingError::NoFieldsToUpdate);
    }


    let response = RequestBuilder::patch(client)
        .endpoint_url(format!(
            "/dictionary/english/{}/meaning/{}",
            english_word_id, english_word_meaning_id
        ))
        .json(&EnglishWordMeaningUpdateRequest {
            abbreviation: fields_to_update.abbreviation,
            disambiguation: fields_to_update.disambiguation,
            description: fields_to_update.description,
        })
        .send()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        let response_body = response.json::<EnglishWordMeaningUpdatedResponse>().await?;

        Ok(response_body.meaning)
    } else if response_status == StatusCode::NOT_FOUND {
        let word_error_reason = response.word_error_reason().await?;

        match word_error_reason {
            WordErrorReason::WordNotFound => Err(EnglishWordMeaningUpdatingError::WordNotFound),
            WordErrorReason::WordMeaningNotFound => {
                Err(EnglishWordMeaningUpdatingError::WordMeaningNotFound)
            }
            _ => unexpected_error_reason!(word_error_reason, response_status),
        }
    } else if response_status == StatusCode::FORBIDDEN {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::MissingPermissions]);
    } else {
        handle_unexpected_status_code!(response_status);
    }
}


async fn delete_english_word_meaning(
    client: &AuthenticatedClient,
    english_word_id: EnglishWordId,
    english_word_meaning_id: EnglishWordMeaningId,
) -> ClientResult<(), EnglishWordMeaningDeletionError> {
    let response = RequestBuilder::delete(client)
        .endpoint_url(format!(
            "/dictionary/english/{}/meaning/{}",
            english_word_id, english_word_meaning_id
        ))
        .send()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        Ok(())
    } else if response_status == StatusCode::NOT_FOUND {
        let word_error_reason = response.word_error_reason().await?;

        match word_error_reason {
            WordErrorReason::WordNotFound => Err(EnglishWordMeaningDeletionError::WordNotFound),
            WordErrorReason::WordMeaningNotFound => {
                Err(EnglishWordMeaningDeletionError::WordMeaningNotFound)
            }
            _ => unexpected_error_reason!(word_error_reason, response_status),
        }
    } else if response_status == StatusCode::FORBIDDEN {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::MissingPermissions]);
    } else {
        handle_unexpected_status_code!(response_status);
    }
}



pub struct EnglishApi<'c> {
    client: &'c Client,
}

impl<'c> EnglishApi<'c> {
    /*
     * Word-related (word meanings are in the next section)
     */
    pub async fn english_words(
        &self,
        options: EnglishWordFetchingOptions,
    ) -> ClientResult<Vec<EnglishWordWithMeanings>> {
        get_english_words(self.client, options).await
    }

    pub async fn english_word_by_id(
        &self,
        english_word_id: EnglishWordId,
    ) -> ClientResult<EnglishWordWithMeanings, EnglishWordFetchingError> {
        get_english_word_by_id(self.client, english_word_id).await
    }

    pub async fn english_word_by_lemma(
        &self,
        english_word_lemma: &str,
    ) -> ClientResult<EnglishWordWithMeanings, EnglishWordFetchingError> {
        get_english_word_by_lemma(self.client, english_word_lemma).await
    }

    /*
     * Word meaning-related (words themselves are in the previous section)
     */
    pub async fn english_word_meanings(
        &self,
        english_word_id: EnglishWordId,
    ) -> ClientResult<
        Vec<EnglishWordMeaningWithCategoriesAndTranslations>,
        EnglishWordMeaningsFetchingError,
    > {
        get_english_word_meanings(self.client, english_word_id).await
    }
}




pub struct EnglishAuthenticatedApi<'c> {
    client: &'c AuthenticatedClient,
}

impl<'c> EnglishAuthenticatedApi<'c> {
    /*
     * Word-related (word meanings are in the next section)
     */
    pub async fn english_words(
        &self,
        options: EnglishWordFetchingOptions,
    ) -> ClientResult<Vec<EnglishWordWithMeanings>> {
        get_english_words(self.client, options).await
    }

    pub async fn english_word_by_id(
        &self,
        english_word_id: EnglishWordId,
    ) -> ClientResult<EnglishWordWithMeanings, EnglishWordFetchingError> {
        get_english_word_by_id(self.client, english_word_id).await
    }

    pub async fn english_word_by_lemma(
        &self,
        english_word_lemma: &str,
    ) -> ClientResult<EnglishWordWithMeanings, EnglishWordFetchingError> {
        get_english_word_by_lemma(self.client, english_word_lemma).await
    }

    pub async fn create_english_word(
        &self,
        word: EnglishWordToCreate,
    ) -> ClientResult<EnglishWordWithMeanings, EnglishWordCreationError> {
        create_english_word(self.client, word).await
    }

    pub async fn update_english_word(
        &self,
        english_word_id: EnglishWordId,
        fields_to_update: EnglishWordFieldsToUpdate,
    ) -> ClientResult<EnglishWordWithMeanings, EnglishWordUpdatingError> {
        update_english_word(self.client, english_word_id, fields_to_update).await
    }

    pub async fn delete_english_word(
        &self,
        english_word_id: EnglishWordId,
    ) -> ClientResult<(), EnglishWordDeletionError> {
        delete_english_word(self.client, english_word_id).await
    }

    /*
     * Word meaning-related (words themselves are in the previous section)
     */
    pub async fn english_word_meanings(
        &self,
        english_word_id: EnglishWordId,
    ) -> ClientResult<
        Vec<EnglishWordMeaningWithCategoriesAndTranslations>,
        EnglishWordMeaningsFetchingError,
    > {
        get_english_word_meanings(self.client, english_word_id).await
    }

    pub async fn create_english_word_meaning(
        &self,
        english_word_id: EnglishWordId,
        word_meaning_to_create: EnglishWordMeaningToCreate,
    ) -> ClientResult<EnglishWordMeaning, EnglishWordMeaningCreationError> {
        create_english_word_meaning(
            self.client,
            english_word_id,
            word_meaning_to_create,
        )
        .await
    }

    pub async fn update_english_word_meaning(
        &self,
        english_word_id: EnglishWordId,
        english_word_meaning_id: EnglishWordMeaningId,
        fields_to_update: EnglishWordMeaningFieldsToUpdate,
    ) -> ClientResult<EnglishWordMeaningWithCategoriesAndTranslations, EnglishWordMeaningUpdatingError>
    {
        update_english_word_meaning(
            self.client,
            english_word_id,
            english_word_meaning_id,
            fields_to_update,
        )
        .await
    }

    pub async fn delete_english_word_meaning(
        &self,
        english_word_id: EnglishWordId,
        english_word_meaning_id: EnglishWordMeaningId,
    ) -> ClientResult<(), EnglishWordMeaningDeletionError> {
        delete_english_word_meaning(
            self.client,
            english_word_id,
            english_word_meaning_id,
        )
        .await
    }
}
