use kolomoni_core::{
    api_models::{
        CategoriesResponse,
        Category,
        CategoryCreationRequest,
        CategoryCreationResponse,
        CategoryErrorReason,
        CategoryResponse,
        CategoryUpdateRequest,
    },
    ids::CategoryId,
};
use reqwest::StatusCode;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    errors::{ClientError, ClientResult},
    macros::{
        handle_error_reasons_or_catch_unexpected_status,
        handlers,
        internal_server_error,
        unexpected_error_reason,
        unexpected_status_code,
    },
    request::RequestBuilder,
    AuthenticatedClient,
    Client,
    HttpClient,
};



pub struct CategoryToCreate {
    pub parent_category_id: Option<CategoryId>,

    pub slovene_category_name: String,

    pub english_category_name: String,
}



pub struct CategoryFieldsToUpdate {
    pub new_parent_category_id: Option<Option<Uuid>>,

    pub new_slovene_name: Option<String>,

    pub new_english_name: Option<String>,
}

impl CategoryFieldsToUpdate {
    pub(crate) fn has_no_fields_to_update(&self) -> bool {
        self.new_parent_category_id.is_none()
            && self.new_slovene_name.is_none()
            && self.new_english_name.is_none()
    }
}




#[derive(Debug, Error)]
pub enum CategoryCreationError {
    #[error("category with this english name already exists")]
    EnglishNameAlreadyExists,

    #[error("category with this slovene name already exists")]
    SloveneNameAlreadyExists,

    #[error(transparent)]
    ClientError {
        #[from]
        error: ClientError,
    },
}


#[derive(Debug, Error)]
pub enum CategoryFetchingError {
    #[error("category does not exist")]
    NotFound,

    #[error(transparent)]
    ClientError {
        #[from]
        error: ClientError,
    },
}


#[derive(Debug, Error)]
pub enum CategoryUpdatingError {
    #[error("category does not exist")]
    NotFound,

    #[error("category with this english name already exists")]
    EnglishNameAlreadyExists,

    #[error("category with this slovene name already exists")]
    SloveneNameAlreadyExists,

    #[error("there were no fields to update")]
    NoFieldsToUpdate,

    #[error(transparent)]
    ClientError {
        #[from]
        error: ClientError,
    },
}


#[derive(Debug, Error)]
pub enum CategoryDeletionError {
    #[error("category does not exist")]
    NotFound,

    #[error(transparent)]
    ClientError {
        #[from]
        error: ClientError,
    },
}



async fn get_categories<C>(client: &C) -> ClientResult<Vec<Category>>
where
    C: HttpClient,
{
    let response = RequestBuilder::get(client)
        .endpoint_url("/dictionary/category")
        .send()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        let response_data = response.json::<CategoriesResponse>().await?;

        Ok(response_data.categories)
    } else if response_status == StatusCode::FORBIDDEN {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::MissingPermissions]);
    } else if response_status == StatusCode::INTERNAL_SERVER_ERROR {
        internal_server_error!()
    } else {
        unexpected_status_code!(response_status)
    }
}


async fn get_category_by_id<C>(
    client: &C,
    category_id: CategoryId,
) -> ClientResult<Category, CategoryFetchingError>
where
    C: HttpClient,
{
    let response = RequestBuilder::get(client)
        .endpoint_url(format!("/dictionary/category/{}", category_id))
        .send()
        .await?;

    let response_status = response.status();

    if response_status == StatusCode::OK {
        let response_data = response.json::<CategoryResponse>().await?;

        Ok(response_data.category)
    } else if response_status == StatusCode::NOT_FOUND {
        let category_error_reason = response.json_category_error_reason().await?;

        match category_error_reason {
            CategoryErrorReason::CategoryNotFound => Err(CategoryFetchingError::NotFound),
            _ => Err(ClientError::unexpected_error_reason(
                category_error_reason.into(),
                response_status,
            )
            .into()),
        }
    } else if response_status == StatusCode::BAD_REQUEST {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::InvalidUuidFormat]);
    } else if response_status == StatusCode::FORBIDDEN {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::MissingPermissions]);
    } else {
        unexpected_status_code!(response_status)
    }
}


async fn update_category(
    client: &AuthenticatedClient,
    category_id: CategoryId,
    category_fields_to_update: CategoryFieldsToUpdate,
) -> ClientResult<Category, CategoryUpdatingError> {
    if category_fields_to_update.has_no_fields_to_update() {
        return Err(CategoryUpdatingError::NoFieldsToUpdate);
    }


    let response = RequestBuilder::patch(client)
        .endpoint_url(format!("/dictionary/category/{}", category_id))
        .json(&CategoryUpdateRequest {
            new_parent_category_id: category_fields_to_update.new_parent_category_id,
            new_english_name: category_fields_to_update.new_english_name,
            new_slovene_name: category_fields_to_update.new_slovene_name,
        })
        .send()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        let response_data = response.json::<CategoryResponse>().await?;

        Ok(response_data.category)
    } else if response_status == StatusCode::BAD_REQUEST {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::InvalidUuidFormat]);
    } else if response_status == StatusCode::NOT_FOUND {
        let category_error_reason = response.json_category_error_reason().await?;

        match category_error_reason {
            CategoryErrorReason::CategoryNotFound => Err(CategoryUpdatingError::NotFound),
            _ => unexpected_error_reason!(category_error_reason, response_status),
        }
    } else if response_status == StatusCode::CONFLICT {
        let category_error_reason = response.json_category_error_reason().await?;

        match category_error_reason {
            CategoryErrorReason::SloveneNameAlreadyExists => {
                Err(CategoryUpdatingError::SloveneNameAlreadyExists)
            }
            CategoryErrorReason::EnglishNameAlreadyExists => {
                Err(CategoryUpdatingError::EnglishNameAlreadyExists)
            }
            _ => unexpected_error_reason!(category_error_reason, response_status),
        }
    } else if response_status == StatusCode::FORBIDDEN {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::MissingPermissions]);
    } else if response_status == StatusCode::INTERNAL_SERVER_ERROR {
        internal_server_error!()
    } else {
        unexpected_status_code!(response_status)
    }
}


async fn create_category(
    client: &AuthenticatedClient,
    category: CategoryToCreate,
) -> ClientResult<Category, CategoryCreationError> {
    let response = RequestBuilder::post(client)
        .endpoint_url("/dictionary/category")
        .json(&CategoryCreationRequest {
            parent_category_id: category.parent_category_id.map(CategoryId::into_uuid),
            english_name: category.english_category_name,
            slovene_name: category.slovene_category_name,
        })
        .send()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        let response_data = response.json::<CategoryCreationResponse>().await?;

        Ok(response_data.category)
    } else if response_status == StatusCode::CONFLICT {
        let category_error_reason = response.json_category_error_reason().await?;

        match category_error_reason {
            CategoryErrorReason::EnglishNameAlreadyExists => {
                Err(CategoryCreationError::EnglishNameAlreadyExists)
            }
            CategoryErrorReason::SloveneNameAlreadyExists => {
                Err(CategoryCreationError::SloveneNameAlreadyExists)
            }
            _ => unexpected_error_reason!(category_error_reason, response_status),
        }
    } else if response_status == StatusCode::FORBIDDEN {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::MissingPermissions]);
    } else if response_status == StatusCode::INTERNAL_SERVER_ERROR {
        internal_server_error!();
    } else {
        unexpected_status_code!(response_status);
    }
}


async fn delete_category(
    client: &AuthenticatedClient,
    category_id: CategoryId,
) -> ClientResult<(), CategoryDeletionError> {
    let response = RequestBuilder::post(client)
        .endpoint_url(format!(
            "/dictionary/category/{}",
            category_id.into_uuid()
        ))
        .send()
        .await?;

    let response_status = response.status();


    if response_status == StatusCode::OK {
        Ok(())
    } else if response_status == StatusCode::NOT_FOUND {
        let category_error_response = response.json_category_error_reason().await?;

        match category_error_response {
            CategoryErrorReason::CategoryNotFound => Err(CategoryDeletionError::NotFound),
            _ => unexpected_error_reason!(category_error_response, response_status),
        }
    } else if response_status == StatusCode::BAD_REQUEST {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::InvalidUuidFormat]);
    } else if response_status == StatusCode::FORBIDDEN {
        handle_error_reasons_or_catch_unexpected_status!(response, [handlers::MissingPermissions]);
    } else if response_status == StatusCode::INTERNAL_SERVER_ERROR {
        internal_server_error!();
    } else {
        unexpected_status_code!(response_status);
    }
}



pub struct DictionaryCategoriesApi<'c> {
    client: &'c Client,
}


impl<'c> DictionaryCategoriesApi<'c> {
    pub async fn get_categories(&self) -> ClientResult<Vec<Category>, ClientError> {
        get_categories(self.client).await
    }

    pub async fn get_category_by_id<C>(
        &self,
        category_id: CategoryId,
    ) -> ClientResult<Category, CategoryFetchingError> {
        get_category_by_id(self.client, category_id).await
    }
}



pub struct DictionaryCategoriesAuthenticatedApi<'c> {
    client: &'c AuthenticatedClient,
}

impl<'c> DictionaryCategoriesAuthenticatedApi<'c> {
    pub async fn get_categories(&self) -> ClientResult<Vec<Category>, ClientError> {
        get_categories(self.client).await
    }

    pub async fn get_category_by_id<C>(
        &self,
        category_id: CategoryId,
    ) -> ClientResult<Category, CategoryFetchingError> {
        get_category_by_id(self.client, category_id).await
    }

    pub async fn update_category(
        &self,
        category_id: CategoryId,
        category_fields_to_update: CategoryFieldsToUpdate,
    ) -> ClientResult<Category, CategoryUpdatingError> {
        update_category(
            self.client,
            category_id,
            category_fields_to_update,
        )
        .await
    }

    pub async fn create_category(
        &self,
        category: CategoryToCreate,
    ) -> ClientResult<Category, CategoryCreationError> {
        create_category(self.client, category).await
    }

    pub async fn delete_category(
        &self,
        category_id: CategoryId,
    ) -> ClientResult<(), CategoryDeletionError> {
        delete_category(self.client, category_id).await
    }
}
