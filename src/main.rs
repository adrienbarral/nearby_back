use actix_web::{web, App, HttpServer};

mod routes;
mod models;
mod database;
use routes::user_available;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/user_available", web::post().to(user_available::user_available))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}