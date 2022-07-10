use actix_web::{get, post, delete, HttpResponse, HttpRequest, patch};

#[post("")]
async fn register_user() -> HttpResponse {
    println!("Accessing users");
    HttpResponse::Ok()
    .body("TODO")
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
