use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder, Scope};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::api::auth::UserAuth;
use crate::database::mutation::users::{Mutation, UserRegistrationInfo};
use crate::database::queries::users;
use crate::database::{entities, queries};
use crate::impl_json_responder_on_serializable;
use crate::state::AppState;

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

#[post("/")]
pub async fn register_user(
    state: web::Data<AppState>,
    json_data: web::Json<UserRegistrationData>,
) -> impl Responder {
    let user_creation_result = Mutation::create_user(
        &state.database,
        &state.hasher,
        json_data.clone().into(),
    )
    .await;

    match user_creation_result {
        Ok(_) => {
            debug!(
                username = json_data.username,
                "User has registered."
            );

            HttpResponse::Ok()
                .content_type(ContentType::json())
                .finish()
        }
        Err(error) => {
            error!(
                error = error.to_string(),
                username = json_data.username,
                "Failed to register user!"
            );

            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .finish()
        }
    }
}


/*
 * GET /me
 */

#[derive(Serialize, Debug)]
pub struct PublicUserModel {
    pub username: String,
    pub display_name: String,
    pub joined_at: DateTime<Utc>,
    pub last_modified_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
}

impl PublicUserModel {
    #[inline]
    pub fn from_seaorm_model(model: entities::users::Model) -> Self {
        Self {
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
            user: PublicUserModel::from_seaorm_model(model),
        }
    }
}

impl_json_responder_on_serializable!(UserInfoResponse, "UserInfoResponse");


#[get("/me")]
pub async fn get_current_user_info(
    request: HttpRequest,
    user_auth: UserAuth,
    state: web::Data<AppState>,
) -> impl Responder {
    let Some(token) = user_auth.auth_token() else {
        return HttpResponse::Forbidden().finish();
    };

    let optional_user =
        match users::Query::get_user_by_username(&state.database, &token.username).await {
            Ok(optional_user) => optional_user,
            Err(error) => {
                error!(
                    error = error.to_string(),
                    username = token.username,
                    "Errored while looking up user by username."
                );

                return HttpResponse::InternalServerError().finish();
            }
        };

    match optional_user {
        Some(user) => UserInfoResponse::new(user).respond_to(&request),
        None => HttpResponse::Gone().finish(),
    }
}

/*
 * GET /me/permissions
 */

#[derive(Serialize, Debug)]
pub struct UserPermissionsResponse {
    pub permissions: Vec<String>,
}

impl_json_responder_on_serializable!(UserPermissionsResponse, "UserPermissionsResponse");


#[get("/me/permissions")]
async fn get_current_user_permissions(
    request: HttpRequest,
    user_auth: UserAuth,
    state: web::Data<AppState>,
) -> impl Responder {
    let Some(token) = user_auth.auth_token() else {
        return HttpResponse::Forbidden().finish();
    };

    let permissions = match queries::user_permissions::Query::get_user_permissions_by_username(
        &state.database,
        &token.username,
    )
    .await
    {
        Ok(optional_permissions) => match optional_permissions {
            Some(permissions) => permissions,
            None => {
                error!(
                    username = token.username,
                    "Failed to get user permissions - user with this token doesn't exist!"
                );

                return HttpResponse::InternalServerError().finish();
            }
        },
        Err(error) => {
            error!(
                error = error.to_string(),
                username = token.username,
                "Errored while getting user permissions."
            );

            return HttpResponse::InternalServerError().finish();
        }
    };

    UserPermissionsResponse { permissions }.respond_to(&request)
}

pub fn users_router() -> Scope {
    web::scope("users")
        .service(register_user)
        .service(get_current_user_info)
        .service(get_current_user_permissions)
}
