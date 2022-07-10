use actix_web::{get, post, delete, HttpResponse, HttpRequest, patch, web};
use crate::{api::v1::users::schemas::UserRegister, errors::BackendError};

#[post("")]
async fn register_user(user: web::Json<UserRegister>) -> Result<HttpResponse, BackendError> {
    if user.display_name.is_some() {
        println!("  Also goes by the name {}.", user.display_name.as_ref().unwrap())
    }
    Ok(HttpResponse::Ok().body("TODO"))
}

#[post("/login")]
async fn login_user() -> HttpResponse {
    HttpResponse::Ok()
    .body("TODO")
}

#[patch("/login")]
async fn refresh_user_token() -> HttpResponse {
    HttpResponse::Ok()
    .body("TODO")
}

#[get("/me")]
async fn retrieve_current_user() -> HttpResponse {
    HttpResponse::Ok()
    .body("TODO")
}

#[patch("/me")]
async fn update_current_user() -> HttpResponse {
    HttpResponse::Ok()
    .body("TODO")
}

#[delete("/me")]
async fn delete_current_user() -> HttpResponse {
    HttpResponse::Ok()
    .body("TODO")
}

#[get("/{id}")]
async fn retrieve_user(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok()
    .body("TODO")
}
