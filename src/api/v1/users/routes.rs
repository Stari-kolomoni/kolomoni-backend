use actix_web::{get, post, delete, HttpResponse, HttpRequest, patch, web};
use crate::Pool;
use crate::api::v1::users::schemas::UserRegister;
use crate::api::v1::users::models::User;

#[post("")]
async fn register_user(
    pool: web::Data<Pool>,
    user: web::Json<UserRegister>) -> HttpResponse {
        
    let user_form = user.into_inner();

    let connection = pool.get().expect("Connection not established!");

    return HttpResponse::Created().finish();
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
