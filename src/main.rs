use actix::prelude::*;
use actix_web::{web, App, HttpServer};

mod database;
mod models;
mod routes;
use database::{
    available_users_cleaner::AvailableUserCleaner, database_interface::DataBaseInterface,
};
use routes::user_available;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // If we can't create database interface here, this is unrecoverable !
    let database_interface = DataBaseInterface::new().await.unwrap();
    let user_cleaner = AvailableUserCleaner::new(database_interface.clone());
    user_cleaner.start();

    HttpServer::new(move || {
        App::new()
            .data(database_interface.clone())
            .route(
                "/user_available",
                web::post().to(user_available::user_available),
            )
            .route(
                "/contacts_availables_nearby",
                web::get().to(user_available::get_nearby_friends),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
