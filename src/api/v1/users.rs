use actix_web::http::header::ContentType;
use actix_web::{post, web, HttpResponse, Responder, Scope};
use serde::Deserialize;
use tracing::{debug, error};

use crate::database::mutation::users::{Mutation, UserRegistrationInfo};
use crate::state::AppState;

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

// TODO
pub fn users_router() -> Scope {
    web::scope("users").service(register_user)
}
