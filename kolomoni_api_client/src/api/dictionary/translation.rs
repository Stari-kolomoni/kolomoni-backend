use kolomoni_core::{
    api_models::{ErrorReason, TranslationCreationRequest, TranslationsErrorReason},
    ids::{EnglishWordMeaningId, SloveneWordMeaningId},
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
    Client,
};
use crate::{request::ApiClientRequestBuild, AuthenticatedHttpClient};


pub struct TranslationRelationshipToCreate {
    pub english_word_meaning: EnglishWordMeaningId,
    pub slovene_word_meaning: SloveneWordMeaningId,
}


pub struct TranslationRelationshipToDelete {
    pub english_word_meaning: EnglishWordMeaningId,
    pub slovene_word_meaning: SloveneWordMeaningId,
}



#[derive(Debug, Error)]
pub enum TranslationRelationshipCreationError {
    #[error("the provided english word meaning does not exist")]
    EnglishWordMeaningNotFound,

    #[error("the provided slovene word meaning does not exist")]
    SloveneWordMeaningNotFound,

    #[error("the corresponding translation relationship already exists")]
    RelationshipAlreadyExists,

    #[error(transparent)]
    ClientError {
        #[from]
        error: ClientError,
    },
}


#[derive(Debug, Error)]
pub enum TranslationRelationshipDeletionError {
    #[error("the provided english word meaning does not exist")]
    EnglishWordMeaningNotFound,

    #[error("the provided slovene word meaning does not exist")]
    SloveneWordMeaningNotFound,

    #[error("the corresponding translation relationship does not exist")]
    RelationshipNotFound,

    #[error(transparent)]
    ClientError {
        #[from]
        error: ClientError,
    },
}



async fn create_translation_relationship<C>(
    client: &C,
    translation_relationship_to_create: TranslationRelationshipToCreate,
) -> ClientResult<(), TranslationRelationshipCreationError>
where
    C: AuthenticatedHttpClient,
{
    let response = client
        .post_request_builder()
        .endpoint_url("/dictionary/translations")
        .json(&TranslationCreationRequest {
            english_word_meaning_id: translation_relationship_to_create
                .english_word_meaning
                .into_uuid(),
            slovene_word_meaning_id: translation_relationship_to_create
                .slovene_word_meaning
                .into_uuid(),
        })
        .send_authenticated()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        Ok(())
    } else if response_status == StatusCode::BAD_REQUEST {
        let error_reason = response.error_reason().await?;

        match error_reason {
            ErrorReason::Translations(translations_error_reason) => {
                match translations_error_reason {
                    TranslationsErrorReason::EnglishWordMeaningNotFound => {
                        Err(TranslationRelationshipCreationError::EnglishWordMeaningNotFound)
                    }
                    TranslationsErrorReason::SloveneWordMeaningNotFound => {
                        Err(TranslationRelationshipCreationError::SloveneWordMeaningNotFound)
                    }
                    _ => handle_unexpected_error_reason!(translations_error_reason, response_status),
                }
            }
            _ => handle_unexpected_error_reason!(error_reason, response_status),
        }
    } else if response_status == StatusCode::CONFLICT {
        let translations_error_reason = response.translations_error_reason().await?;

        match translations_error_reason {
            TranslationsErrorReason::TranslationRelationshipAlreadyExists => {
                Err(TranslationRelationshipCreationError::RelationshipAlreadyExists)
            }
            _ => handle_unexpected_error_reason!(translations_error_reason, response_status),
        }
    } else if response_status == StatusCode::FORBIDDEN {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::MissingPermissions]);
    } else {
        handle_uncaught_status_code!(response_status);
    }
}


async fn delete_translation_relationship<C>(
    client: &C,
    translation_relationship_to_delete: TranslationRelationshipToDelete,
) -> ClientResult<(), TranslationRelationshipDeletionError>
where
    C: AuthenticatedHttpClient,
{
    let response = client
        .delete_request_builder()
        .endpoint_url_with_parameters(
            "/dictionary/translations",
            [
                (
                    "english_word_meaning_id",
                    translation_relationship_to_delete
                        .english_word_meaning
                        .to_string(),
                ),
                (
                    "slovene_word_meaning_id",
                    translation_relationship_to_delete
                        .slovene_word_meaning
                        .to_string(),
                ),
            ],
        )
        .send_authenticated()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        Ok(())
    } else if response_status == StatusCode::BAD_REQUEST {
        let translation_error_reason = response.translations_error_reason().await?;

        match translation_error_reason {
            TranslationsErrorReason::EnglishWordMeaningNotFound => {
                Err(TranslationRelationshipDeletionError::EnglishWordMeaningNotFound)
            }
            TranslationsErrorReason::SloveneWordMeaningNotFound => {
                Err(TranslationRelationshipDeletionError::SloveneWordMeaningNotFound)
            }
            _ => handle_unexpected_error_reason!(translation_error_reason, response_status),
        }
    } else if response_status == StatusCode::NOT_FOUND {
        let translation_error_reason = response.translations_error_reason().await?;

        match translation_error_reason {
            TranslationsErrorReason::TranslationRelationshipNotFound => {
                Err(TranslationRelationshipDeletionError::RelationshipNotFound)
            }
            _ => handle_unexpected_error_reason!(translation_error_reason, response_status),
        }
    } else if response_status == StatusCode::FORBIDDEN {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::MissingPermissions]);
    } else {
        handle_uncaught_status_code!(response_status);
    }
}


pub struct TranslationsAuthenticatedApi<'c, C>
where
    C: Client + AuthenticatedHttpClient,
{
    client: &'c C,
}

impl<'c, C> TranslationsAuthenticatedApi<'c, C>
where
    C: Client + AuthenticatedHttpClient,
{
    pub(crate) const fn new(client: &'c C) -> Self {
        Self { client }
    }


    pub async fn create_translation_relationship(
        &self,
        translation_relationship_to_create: TranslationRelationshipToCreate,
    ) -> ClientResult<(), TranslationRelationshipCreationError> {
        create_translation_relationship(self.client, translation_relationship_to_create).await
    }

    pub async fn delete_translation_relationship(
        &self,
        translation_relationship_to_delete: TranslationRelationshipToDelete,
    ) -> ClientResult<(), TranslationRelationshipDeletionError> {
        delete_translation_relationship(self.client, translation_relationship_to_delete).await
    }
}
