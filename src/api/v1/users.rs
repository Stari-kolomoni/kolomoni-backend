use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::HttpResponseBuilder;
use actix_web::{get, patch, post, web, HttpRequest, HttpResponse, Responder, Scope};
use chrono::{DateTime, Utc};
use sea_orm::TransactionTrait;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::api::auth::{UserAuth, UserPermission};
use crate::api::errors::{APIError, EndpointResult, ErrorReasonResponse};
use crate::api::macros::DumbResponder;
use crate::database::mutation::users::UserRegistrationInfo;
use crate::database::{entities, mutation, queries};
use crate::state::AppState;
use crate::{impl_json_responder, response_with_reason};

/*
 * Shared
 */

#[derive(Serialize, Debug)]
pub struct PublicUserModel {
    pub id: i32,
    pub username: String,
    pub display_name: String,
    pub joined_at: DateTime<Utc>,
    pub last_modified_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
}

impl PublicUserModel {
    #[inline]
    pub fn from_user_model(model: entities::users::Model) -> Self {
        Self {
            id: model.id,
            username: model.username,
            display_name: model.display_name,
            joined_at: model.joined_at,
            last_modified_at: model.last_modified_at,
            last_active_at: model.last_active_at,
        }
    }
}



#[derive(Serialize, Debug)]
pub struct UserInfoResponse {
    pub user: PublicUserModel,
}

impl UserInfoResponse {
    pub fn new(model: entities::users::Model) -> Self {
        Self {
            user: PublicUserModel::from_user_model(model),
        }
    }
}

impl_json_responder!(UserInfoResponse);



#[derive(Deserialize, Clone, Debug)]
pub struct UserDisplayNameChangeRequest {
    pub new_display_name: String,
}


#[derive(Serialize, Debug)]
pub struct UserDisplayNameChangeResponse {
    pub user: PublicUserModel,
}

impl_json_responder!(UserDisplayNameChangeResponse);


/*
 * POST /
 */

#[derive(Deserialize, Clone, Debug)]
pub struct UserRegistrationData {
    pub username: String,
    pub display_name: String,
    pub password: String,
}

impl From<UserRegistrationData> for UserRegistrationInfo {
    fn from(value: UserRegistrationData) -> Self {
        Self {
            username: value.username,
            display_name: value.display_name,
            password: value.password,
        }
    }
}


#[derive(Serialize, Debug)]
pub struct UserRegistrationResponse {
    pub user: PublicUserModel,
}

impl_json_responder!(UserRegistrationResponse);


#[post("/")]
pub async fn register_user(
    state: web::Data<AppState>,
    json_data: web::Json<UserRegistrationData>,
) -> EndpointResult {
    let username_already_exists =
        queries::users::Query::user_exists_by_username(&state.database, &json_data.username)
            .await
            .map_err(APIError::InternalError)?;

    if username_already_exists {
        return Ok(response_with_reason!(
            StatusCode::CONFLICT,
            "User with provided username already exists."
        ));
    }


    let new_user = mutation::users::Mutation::create_user(
        &state.database,
        &state.hasher,
        json_data.clone().into(),
    )
    .await
    .map_err(APIError::InternalError)?;

    Ok(UserRegistrationResponse {
        user: PublicUserModel::from_user_model(new_user),
    }
    .into_response())
}


/*
 * GET /me
 */


#[get("/me")]
pub async fn get_current_user_info(
    user_auth: UserAuth,
    state: web::Data<AppState>,
) -> EndpointResult {
    let token = user_auth
        .token_if_authenticated()
        .ok_or_else(|| APIError::NotAuthenticated)?;

    let optional_user =
        queries::users::Query::get_user_by_username(&state.database, &token.username)
            .await
            .map_err(APIError::InternalError)?;

    let Some(user) = optional_user else {
        return Ok(HttpResponse::NotFound().finish());
    };

    Ok(UserInfoResponse::new(user).into_response())
}

/*
 * GET /me/permissions
 */

#[derive(Serialize, Debug)]
pub struct UserPermissionsResponse {
    pub permissions: Vec<String>,
}

impl_json_responder!(UserPermissionsResponse);


#[get("/me/permissions")]
async fn get_current_user_permissions(
    user_auth: UserAuth,
    state: web::Data<AppState>,
) -> EndpointResult {
    let token = user_auth
        .token_if_authenticated()
        .ok_or_else(|| APIError::NotAuthenticated)?;

    let user_permissions = queries::user_permissions::Query::get_user_permission_names_by_username(
        &state.database,
        &token.username,
    )
    .await
    .map_err(APIError::InternalError)?
    .ok_or_else(|| {
        APIError::internal_reason(
            "Failed to get user permissions, user with this token doesn't exist!",
        )
    })?;

    Ok(UserPermissionsResponse {
        permissions: user_permissions,
    }
    .into_response())
}

/*
 * PATCH /me/display_name
 */


#[patch("/me/display_name")]
async fn update_current_user_display_name(
    user_auth: UserAuth,
    state: web::Data<AppState>,
    json_data: web::Json<UserDisplayNameChangeRequest>,
) -> EndpointResult {
    // TODO Rate-limiting.

    let token = user_auth
        .token_if_authenticated()
        .ok_or_else(|| APIError::NotAuthenticated)?;

    let json_data = json_data.into_inner();

    let database_transaction = state
        .database
        .begin()
        .await
        .map_err(APIError::InternalDatabaseError)?;

    mutation::users::Mutation::update_display_name_by_username(
        &database_transaction,
        &token.username,
        json_data.new_display_name.clone(),
    )
    .await
    .map_err(APIError::InternalError)?;

    // TODO Consider merging this update into all mutation methods where it makes sense.
    //      Otherwise we're wasting a round-trip to the database for no real reason.
    let updated_user = mutation::users::Mutation::update_last_active_at_by_username(
        &database_transaction,
        &token.username,
        None,
    )
    .await
    .map_err(APIError::InternalError)?;

    database_transaction
        .commit()
        .await
        .map_err(APIError::InternalDatabaseError)?;


    info!(
        username = token.username,
        new_display_name = json_data.new_display_name,
        "User has updated their display name."
    );

    Ok(UserDisplayNameChangeResponse {
        user: PublicUserModel::from_user_model(updated_user),
    }
    .into_response())
}


/*
 * GET /{id}
 */

#[get("/{user_id}")]
async fn get_specific_user_info(
    user_auth: UserAuth,
    path_info: web::Path<(i32,)>,
    state: web::Data<AppState>,
) -> EndpointResult {
    let requested_user_id = path_info.into_inner().0;

    // Only authenticated users with the `user.any:read` permission to access this endpoint.
    let permissions = user_auth
        .permissions_if_authenticated(&state.database)
        .await
        .map_err(APIError::InternalError)?
        .ok_or_else(|| APIError::NotAuthenticated)?;

    if !permissions.has_permission(UserPermission::UserAnyRead) {
        return Err(APIError::NotEnoughPermissions);
    }

    // Return information about the requested user.
    let optional_user = queries::users::Query::get_user_by_id(&state.database, requested_user_id)
        .await
        .map_err(APIError::InternalError)?;

    let Some(user) = optional_user else {
        return Ok(HttpResponse::NotFound().finish());
    };

    Ok(UserInfoResponse::new(user).into_response())
}

/*
 * PATCH /{user_id}/display_name
 */


#[patch("/{user_id}/display_name")]
async fn update_specific_user_display_name(
    user_auth: UserAuth,
    path_info: web::Path<(i32,)>,
    state: web::Data<AppState>,
    json_data: web::Json<UserDisplayNameChangeRequest>,
) -> EndpointResult {
    let requested_user_id = path_info.into_inner().0;

    // Only authenticated users with the `user.any:write` permission can modify
    // others' display names. Intended for moderation tooling.
    let token = user_auth
        .token_if_authenticated()
        .ok_or_else(|| APIError::NotAuthenticated)?;

    let permissions = user_auth
        .permissions_if_authenticated(&state.database)
        .await
        .map_err(APIError::InternalError)?
        .ok_or_else(|| APIError::NotAuthenticated)?;

    if !permissions.has_permission(UserPermission::UserAnyWrite) {
        return Err(APIError::NotEnoughPermissions);
    }

    let json_data = json_data.into_inner();

    let database_transaction = state
        .database
        .begin()
        .await
        .map_err(APIError::InternalDatabaseError)?;

    mutation::users::Mutation::update_display_name_by_user_id(
        &database_transaction,
        requested_user_id,
        json_data.new_display_name.clone(),
    )
    .await
    .map_err(APIError::InternalError)?;

    let updated_user = mutation::users::Mutation::update_last_active_at_by_user_id(
        &database_transaction,
        requested_user_id,
        None,
    )
    .await
    .map_err(APIError::InternalError)?;

    database_transaction
        .commit()
        .await
        .map_err(APIError::InternalDatabaseError)?;


    info!(
        operator = token.username,
        target_user_id = requested_user_id,
        new_display_name = json_data.new_display_name,
        "User has updated another user's display name."
    );

    Ok(UserDisplayNameChangeResponse {
        user: PublicUserModel::from_user_model(updated_user),
    }
    .into_response())
}

/*
 * Router
 */

pub fn users_router() -> Scope {
    web::scope("users")
        .service(register_user)
        .service(get_current_user_info)
        .service(get_current_user_permissions)
        .service(update_current_user_display_name)
        .service(get_specific_user_info)
        .service(update_specific_user_display_name)
}
