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

use crate::{
    api::EndpointGroup,
    client::{errors::RequestError, KolomoniHttpClient},
    parsing::{
        err_if_invalid_uuid,
        err_if_missing_permissions,
        unexpected_error_reason,
        unexpected_response,
    },
    request::{
        typed::{BoundTypedRequest, IntoBoundTypedRequest},
        ToRequestBuilder,
    },
    response::{raw::RawResponse, ResponseValueError},
};



pub struct CategoryToCreate {
    pub parent_category_id: Option<CategoryId>,

    pub slovene_category_name: String,

    pub english_category_name: String,
}



pub struct CategoryFieldsToUpdate {
    pub new_parent_category_id: Option<Option<CategoryId>>,

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
    RequestError(#[from] RequestError),
}

impl ResponseValueError for CategoryCreationError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}


#[derive(Debug, Error)]
pub enum CategoryListError {
    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for CategoryListError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}


#[derive(Debug, Error)]
pub enum CategoryFetchingError {
    #[error("category does not exist")]
    NotFound,

    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for CategoryFetchingError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}


#[derive(Debug, Error)]
pub enum CategoryUpdatePreparationError {
    #[error("there was nothing to update")]
    NothingToUpdate,
}


#[derive(Debug, Error)]
pub enum CategoryUpdateError {
    #[error("category does not exist")]
    NotFound,

    #[error("category with this english name already exists")]
    EnglishNameAlreadyExists,

    #[error("category with this slovene name already exists")]
    SloveneNameAlreadyExists,

    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for CategoryUpdateError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}



#[derive(Debug, Error)]
pub enum CategoryDeletionError {
    #[error("category does not exist")]
    NotFound,

    #[error(transparent)]
    RequestError(#[from] RequestError),
}

impl ResponseValueError for CategoryDeletionError {
    fn from_request_error(error: RequestError) -> Self {
        Self::RequestError(error)
    }
}



fn get_categories_request<'c, C>(
    client: &'c C,
) -> BoundTypedRequest<'c, C, Vec<Category>, CategoryListError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let response_data = response.into_json_body::<CategoriesResponse>().await?;

            Ok(response_data.categories)
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            err_if_missing_permissions::<CategoryListError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .get()
        .endpoint_url("/dictionary/category")
        .build_request()
        .into_bound_typed_request(response_parser)
}




fn get_category_by_id_request<'c, C>(
    client: &'c C,
    category_id: CategoryId,
) -> BoundTypedRequest<'c, C, Category, CategoryFetchingError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let response_data = response.into_json_body::<CategoryResponse>().await?;

            Ok(response_data.category)
        } else if status == StatusCode::NOT_FOUND {
            let category_error_reason = response.category_error_reason().await?;

            match category_error_reason {
                CategoryErrorReason::CategoryNotFound => Err(CategoryFetchingError::NotFound),
                _ => Err(RequestError::unexpected_error_reason(
                    category_error_reason.into(),
                    status,
                )
                .into()),
            }
        } else if status == StatusCode::BAD_REQUEST {
            let error_reason = response.error_reason().await?;

            err_if_invalid_uuid::<CategoryFetchingError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            err_if_missing_permissions::<CategoryFetchingError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .get()
        .endpoint_url(format!("/dictionary/category/{}", category_id))
        .build_request()
        .into_bound_typed_request(response_parser)
}


fn update_category_request<'c, C>(
    client: &'c C,
    category_id: CategoryId,
    update: CategoryFieldsToUpdate,
) -> Result<BoundTypedRequest<'c, C, Category, CategoryUpdateError>, CategoryUpdatePreparationError>
where
    C: KolomoniHttpClient,
{
    if update.has_no_fields_to_update() {
        return Err(CategoryUpdatePreparationError::NothingToUpdate);
    }

    let new_parent_category_id = update
        .new_parent_category_id
        .map(|outer| outer.map(|inner| inner.into_uuid()));


    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let response_data = response.into_json_body::<CategoryResponse>().await?;

            Ok(response_data.category)
        } else if status == StatusCode::BAD_REQUEST {
            let error_reason = response.error_reason().await?;

            err_if_invalid_uuid::<CategoryUpdateError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else if status == StatusCode::NOT_FOUND {
            let category_error_reason = response.category_error_reason().await?;

            match category_error_reason {
                CategoryErrorReason::CategoryNotFound => Err(CategoryUpdateError::NotFound),
                _ => Err(unexpected_error_reason(
                    status,
                    category_error_reason,
                )),
            }
        } else if status == StatusCode::CONFLICT {
            let category_error_reason = response.category_error_reason().await?;

            match category_error_reason {
                CategoryErrorReason::SloveneNameAlreadyExists => {
                    Err(CategoryUpdateError::SloveneNameAlreadyExists)
                }
                CategoryErrorReason::EnglishNameAlreadyExists => {
                    Err(CategoryUpdateError::EnglishNameAlreadyExists)
                }
                _ => Err(unexpected_error_reason(
                    status,
                    category_error_reason,
                )),
            }
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            err_if_missing_permissions::<CategoryUpdateError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else {
            Err(unexpected_response(response).await)
        }
    };

    Ok(client
        .patch()
        .endpoint_url(format!("/dictionary/category/{}", category_id))
        .json(&CategoryUpdateRequest {
            new_parent_category_id,
            new_english_name: update.new_english_name,
            new_slovene_name: update.new_slovene_name,
        })
        .build_request()
        .into_bound_typed_request(response_parser))
}



fn create_category_request<'c, C>(
    client: &'c C,
    category: CategoryToCreate,
) -> BoundTypedRequest<'c, C, Category, CategoryCreationError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            let response_data = response
                .into_json_body::<CategoryCreationResponse>()
                .await?;

            Ok(response_data.category)
        } else if status == StatusCode::CONFLICT {
            let category_error_reason = response.category_error_reason().await?;

            match category_error_reason {
                CategoryErrorReason::EnglishNameAlreadyExists => {
                    Err(CategoryCreationError::EnglishNameAlreadyExists)
                }
                CategoryErrorReason::SloveneNameAlreadyExists => {
                    Err(CategoryCreationError::SloveneNameAlreadyExists)
                }
                _ => Err(unexpected_error_reason(
                    status,
                    category_error_reason,
                )),
            }
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            err_if_missing_permissions::<CategoryCreationError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .post()
        .endpoint_url("/dictionary/category")
        .json(&CategoryCreationRequest {
            parent_category_id: category.parent_category_id.map(CategoryId::into_uuid),
            english_name: category.english_category_name,
            slovene_name: category.slovene_category_name,
        })
        .build_request()
        .into_bound_typed_request(response_parser)
}



fn delete_category_request<'c, C>(
    client: &'c C,
    category_id: CategoryId,
) -> BoundTypedRequest<'c, C, (), CategoryDeletionError>
where
    C: KolomoniHttpClient,
{
    let response_parser = async |response: RawResponse| {
        let status = response.status();

        if status == StatusCode::OK {
            Ok(())
        } else if status == StatusCode::NOT_FOUND {
            let category_error_response = response.category_error_reason().await?;

            match category_error_response {
                CategoryErrorReason::CategoryNotFound => Err(CategoryDeletionError::NotFound),
                _ => Err(unexpected_error_reason(
                    status,
                    category_error_response,
                )),
            }
        } else if status == StatusCode::BAD_REQUEST {
            let error_reason = response.error_reason().await?;

            err_if_invalid_uuid::<CategoryDeletionError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else if status == StatusCode::FORBIDDEN {
            let error_reason = response.error_reason().await?;

            err_if_missing_permissions::<CategoryDeletionError>(status, &error_reason)?;
            Err(unexpected_error_reason(status, error_reason))
        } else {
            Err(unexpected_response(response).await)
        }
    };

    client
        .delete()
        .endpoint_url(format!(
            "/dictionary/category/{}",
            category_id.into_uuid()
        ))
        .build_request()
        .into_bound_typed_request(response_parser)
}



pub trait DictionaryCategoriesUnauthenticatedEndpoints<'c, C>: EndpointGroup<'c, C>
where
    C: KolomoniHttpClient,
{
    fn get_categories(&'c self) -> BoundTypedRequest<'c, C, Vec<Category>, CategoryListError> {
        get_categories_request(self.client())
    }

    fn get_category_by_id(
        &'c self,
        category_id: CategoryId,
    ) -> BoundTypedRequest<'c, C, Category, CategoryFetchingError> {
        get_category_by_id_request(self.client(), category_id)
    }
}

pub trait DictionaryCategoriesAuthenticatedEndpoints<'c, C>: EndpointGroup<'c, C>
where
    C: KolomoniHttpClient,
{
    fn update_category(
        &'c self,
        category_id: CategoryId,
        update: CategoryFieldsToUpdate,
    ) -> Result<
        BoundTypedRequest<'c, C, Category, CategoryUpdateError>,
        CategoryUpdatePreparationError,
    > {
        update_category_request(self.client(), category_id, update)
    }

    fn create_category(
        &'c self,
        category: CategoryToCreate,
    ) -> BoundTypedRequest<'c, C, Category, CategoryCreationError> {
        create_category_request(self.client(), category)
    }

    fn delete_category(
        &'c self,
        category_id: CategoryId,
    ) -> BoundTypedRequest<'c, C, (), CategoryDeletionError> {
        delete_category_request(self.client(), category_id)
    }
}


pub struct DictionaryCategoriesUnauthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
    client: &'c C,
}


impl<'c, C> DictionaryCategoriesUnauthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
    pub(crate) const fn new(client: &'c C) -> Self {
        Self { client }
    }
}

impl<'c, C> EndpointGroup<'c, C> for DictionaryCategoriesUnauthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
    fn client(&'c self) -> &'c C {
        self.client
    }
}

impl<'c, C> DictionaryCategoriesUnauthenticatedEndpoints<'c, C>
    for DictionaryCategoriesUnauthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
}



pub struct DictionaryCategoriesAuthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
    client: &'c C,
}

impl<'c, C> DictionaryCategoriesAuthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
    pub(crate) const fn new(client: &'c C) -> Self {
        Self { client }
    }
}


impl<'c, C> EndpointGroup<'c, C> for DictionaryCategoriesAuthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
    fn client(&'c self) -> &'c C {
        self.client
    }
}

impl<'c, C> DictionaryCategoriesUnauthenticatedEndpoints<'c, C>
    for DictionaryCategoriesAuthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
}

impl<'c, C> DictionaryCategoriesAuthenticatedEndpoints<'c, C>
    for DictionaryCategoriesAuthenticatedApi<'c, C>
where
    C: KolomoniHttpClient,
{
}
