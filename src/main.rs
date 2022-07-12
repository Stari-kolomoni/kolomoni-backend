use std::env;

use actix_web::{get, HttpServer, App, HttpResponse, web};
use diesel::PgConnection;
use r2d2_diesel::ConnectionManager;

mod api;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[get("/ping")]
async fn ping() -> HttpResponse {
    HttpResponse::Ok().json("Pong!")
}

fn create_db_connection() -> Pool {
    let database_url = env::var("DATABASE_URL")
        .expect("Database URL not set!");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Pool creation failed!");
    return pool;
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let address = env::var("KOLOMONI_ADDRESS")
        .expect("Address not set!");
    let port: u16 = env::var("KOLOMONI_PORT")
        .expect("Port not set!").parse().unwrap();

    let db_connection_pool = create_db_connection();

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_connection_pool.clone()))
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
