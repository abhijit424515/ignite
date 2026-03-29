mod api;
mod common;
mod db;
mod embedding;
mod memory;
#[cfg(test)]
mod tests;

use actix_web::{App, HttpServer, web};
use crate::db::surreal::Database;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database = web::Data::new(Database::new().await.map_err(std::io::Error::other)?);

    HttpServer::new(move || {
        App::new()
            .app_data(database.clone())
            .configure(api::configure)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
