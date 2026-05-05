use kolomoni_core::{
    api_models::{ErrorReason, TranslationCreationRequest, TranslationsErrorReason},
    ids::{EnglishWordMeaningId, SloveneWordMeaningId},
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

pub struct TranslationRelationshipToCreate {
    pub english_word_meaning: EnglishWordMeaningId,
    pub slovene_word_meaning: SloveneWordMeaningId,
}


pub struct TranslationRelationshipToDelete {
    pub english_word_meaning: EnglishWordMeaningId,
    pub slovene_word_meaning: SloveneWordMeaningId,
}



#[derive(Debug, Error)]
pub enum TranslationRelationshipCreateError {
    #[error("the provided english word meaning does not exist")]
    EnglishWordMeaningNotFound,

    #[error("the provided slovene word meaning does not exist")]
    SloveneWordMeaningNotFound,

    #[error("the corresponding translation relationship already exists")]
    RelationshipAlreadyExists,

    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for TranslationRelationshipCreateError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
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
    RequestError(#[from] RequestError),
}

impl ResponseValueError for TranslationRelationshipDeletionError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}



fn create_translation_relationship_request<'c, C>(
    client: &'c C,
    translation_relationship_to_create: TranslationRelationshipToCreate,
) -> BoundTypedRequest<'c, C, (), TranslationRelationshipCreateError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            Ok(())
        } else if status == StatusCode::BAD_REQUEST {
            let error_reason = response.error_reason().await?;

            match error_reason {
                ErrorReason::Translations(translations_error_reason) => {
                    match translations_error_reason {
                        TranslationsErrorReason::EnglishWordMeaningNotFound => {
                            Err(TranslationRelationshipCreateError::EnglishWordMeaningNotFound)
                        }
                        TranslationsErrorReason::SloveneWordMeaningNotFound => {
                            Err(TranslationRelationshipCreateError::SloveneWordMeaningNotFound)
                        }
                        _ => Err(unexpected_error_reason(
                            status,
                            translations_error_reason,
                        )),
                    }
                }
                _ => Err(unexpected_error_reason(status, error_reason)),
            }
        } else if status == StatusCode::CONFLICT {
            let translations_error_reason = response.translations_error_reason().await?;

            match translations_error_reason {
                TranslationsErrorReason::TranslationRelationshipAlreadyExists => {
                    Err(TranslationRelationshipCreateError::RelationshipAlreadyExists)
                }
                _ => Err(unexpected_error_reason(
                    status,
                    translations_error_reason,
                )),
            }
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            err_if_missing_permissions::<TranslationRelationshipCreateError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .post()
        .endpoint_url("/dictionary/translations")
        .json(&TranslationCreationRequest {
            english_word_meaning_id: translation_relationship_to_create
                .english_word_meaning
                .into_uuid(),
            slovene_word_meaning_id: translation_relationship_to_create
                .slovene_word_meaning
                .into_uuid(),
        })
        .build_request()
        .into_bound_typed_request(response_parser)
}


fn delete_translation_relationship_request<'c, C>(
    client: &'c C,
    translation_relationship_to_delete: TranslationRelationshipToDelete,
) -> BoundTypedRequest<'c, C, (), TranslationRelationshipDeletionError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            Ok(())
        } else if status == StatusCode::BAD_REQUEST {
            let translation_error_reason = response.translations_error_reason().await?;

            match translation_error_reason {
                TranslationsErrorReason::EnglishWordMeaningNotFound => {
                    Err(TranslationRelationshipDeletionError::EnglishWordMeaningNotFound)
                }
                TranslationsErrorReason::SloveneWordMeaningNotFound => {
                    Err(TranslationRelationshipDeletionError::SloveneWordMeaningNotFound)
                }
                _ => Err(unexpected_error_reason(
                    status,
                    translation_error_reason,
                )),
            }
        } else if status == StatusCode::NOT_FOUND {
            let translation_error_reason = response.translations_error_reason().await?;

            match translation_error_reason {
                TranslationsErrorReason::TranslationRelationshipNotFound => {
                    Err(TranslationRelationshipDeletionError::RelationshipNotFound)
                }
                _ => Err(unexpected_error_reason(
                    status,
                    translation_error_reason,
                )),
            }
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            err_if_missing_permissions::<TranslationRelationshipDeletionError>(
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
        .build_request()
        .into_bound_typed_request(response_parser)
}



pub trait TranslationAuthenticatedEndpoints<'c, C>: EndpointGroup<'c, C>
where
    C: KolomoniHttpClient,
{
    fn create_translation_relationship(
        &'c self,
        translation_relationship_to_create: TranslationRelationshipToCreate,
    ) -> BoundTypedRequest<'c, C, (), TranslationRelationshipCreateError> {
        create_translation_relationship_request(self.client(), translation_relationship_to_create)
    }

    fn delete_translation_relationship(
        &'c self,
        translation_relationship_to_delete: TranslationRelationshipToDelete,
    ) -> BoundTypedRequest<'c, C, (), TranslationRelationshipDeletionError> {
        delete_translation_relationship_request(self.client(), translation_relationship_to_delete)
    }
}



pub struct TranslationsAuthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
    client: &'c C,
}

impl<'c, C> TranslationsAuthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
    pub(crate) const fn new(client: &'c C) -> Self {
        Self { client }
    }
}

impl<'c, C> EndpointGroup<'c, C> for TranslationsAuthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
    fn client(&'c self) -> &'c C {
        self.client
    }
}

impl<'c, C> TranslationAuthenticatedEndpoints<'c, C> for TranslationsAuthenticatedApi<'c, C> where
    C: KolomoniHttpClient
{
}
