use actix_web::body::BoxBody;
use actix_web::{get, patch, post, web, HttpRequest, HttpResponse, Responder, Scope};
use chrono::{DateTime, Utc};
use sea_orm::TransactionTrait;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::api::auth::{UserAuth, UserPermission};
use crate::api::errors::{APIError, EndpointResult, ErrorReasonResponse};
use crate::database::mutation::users::UserRegistrationInfo;
use crate::database::{entities, mutation, queries};
use crate::impl_json_responder;
use crate::state::AppState;

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

impl_json_responder!(UserInfoResponse, "UserInfoResponse");


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

impl_json_responder!(
    UserRegistrationResponse,
    "UserRegistrationResponse"
);


#[post("/")]
pub async fn register_user(
    request: HttpRequest,
    state: web::Data<AppState>,
    json_data: web::Json<UserRegistrationData>,
) -> EndpointResult {
    let username_already_exists =
        queries::users::Query::user_exists_by_username(&state.database, &json_data.username)
            .await
            .map_err(APIError::InternalError)?;

    if username_already_exists {
        return Ok(
            HttpResponse::Conflict().json(ErrorReasonResponse::custom_reason(
                "User with provided username already exists.",
            )),
        );
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
    .respond_to(&request))
}


/*
 * GET /me
 */


#[get("/me")]
pub async fn get_current_user_info(
    request: HttpRequest,
    user_auth: UserAuth,
    state: web::Data<AppState>,
) -> EndpointResult {
    let token = user_auth
        .auth_token()
        .ok_or_else(|| APIError::NotAuthenticated)?;

    let optional_user =
        queries::users::Query::get_user_by_username(&state.database, &token.username)
            .await
            .map_err(APIError::InternalError)?;

    let Some(user) = optional_user else {
        return Ok(HttpResponse::NotFound().finish());
    };

    Ok(UserInfoResponse::new(user).respond_to(&request))
}

/*
 * GET /me/permissions
 */

#[derive(Serialize, Debug)]
pub struct UserPermissionsResponse {
    pub permissions: Vec<String>,
}

impl_json_responder!(UserPermissionsResponse, "UserPermissionsResponse");


#[get("/me/permissions")]
async fn get_current_user_permissions(
    request: HttpRequest,
    user_auth: UserAuth,
    state: web::Data<AppState>,
) -> EndpointResult {
    let token = user_auth
        .auth_token()
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
    .respond_to(&request))
}

/*
 * PATCH /me/display_name
 */

#[derive(Deserialize, Clone, Debug)]
pub struct UserDisplayNameChangeRequest {
    pub new_display_name: String,
}


#[derive(Serialize, Debug)]
pub struct UserDisplayNameChangeResponse {
    pub user: PublicUserModel,
}

impl_json_responder!(
    UserDisplayNameChangeResponse,
    "UserDisplayNameChangeResponse"
);


#[patch("/me/display_name")]
async fn update_username(
    request: HttpRequest,
    user_auth: UserAuth,
    state: web::Data<AppState>,
    json_data: web::Json<UserDisplayNameChangeRequest>,
) -> EndpointResult {
    // TODO Rate-limiting.

    let token = user_auth
        .auth_token()
        .ok_or_else(|| APIError::NotAuthenticated)?;

    let json_data = json_data.into_inner();

    let database_transaction = state
        .database
        .begin()
        .await
        .map_err(APIError::InternalDatabaseError)?;

    let updated_user = mutation::users::Mutation::update_display_name_by_username(
        &database_transaction,
        &token.username,
        json_data.new_display_name.clone(),
    )
    .await
    .map_err(APIError::InternalError)?;

    info!(
        username = token.username,
        new_display_name = json_data.new_display_name,
        "User has updated their display name."
    );

    mutation::users::Mutation::update_last_active_at_by_username(
        &database_transaction,
        &token.username,
        None,
    )
    .await
    .map_err(APIError::InternalError)?;

    Ok(UserDisplayNameChangeResponse {
        user: PublicUserModel::from_user_model(updated_user),
    }
    .respond_to(&request))
}


/*
 * GET /{id}
 */

#[get("/{user_id}")]
async fn get_specific_user_info(
    request: HttpRequest,
    user_auth: UserAuth,
    path_info: web::Path<(i32,)>,
    state: web::Data<AppState>,
) -> EndpointResult {
    let requested_user_id = path_info.into_inner().0;

    // Only authenticated users with the `user.any:read` permission to access this endpoint.
    let permissions = user_auth
        .permissions(&state.database)
        .await
        .map_err(APIError::InternalError)?
        .ok_or_else(|| APIError::NotAuthenticated)?;

    if !permissions.has_permission(UserPermission::UserRead) {
        return Err(APIError::NotEnoughPermissions);
    }

    // Return information about the requested user.
    let optional_user = queries::users::Query::get_user_by_id(&state.database, requested_user_id)
        .await
        .map_err(APIError::InternalError)?;

    let Some(user) = optional_user else {
        return Ok(HttpResponse::NotFound().finish());
    };

    Ok(UserInfoResponse::new(user).respond_to(&request))
}

/*
 * Router
 */

pub fn users_router() -> Scope {
    web::scope("users")
        .service(register_user)
        .service(get_current_user_info)
        .service(get_current_user_permissions)
        .service(update_username)
        .service(get_specific_user_info)
}
