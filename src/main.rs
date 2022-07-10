use actix_web::{get, HttpServer, App, HttpResponse};

mod api;

#[get("/ping")]
async fn ping() -> HttpResponse {
    HttpResponse::Ok().json("Pong!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let address = "localhost";
    let port = 8088;

    let server = HttpServer::new(|| {
        App::new()
            .service(api::api_router())
            .service(ping)
    })
    .bind((address, port));
    
    match server {
        Ok(a) => {
            println!("Starting server at port {port}");
            a.run().await
        },
        Err(err) => panic!("Problem starting server: {:?}", err)
    }
}
