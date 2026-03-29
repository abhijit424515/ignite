mod api;
mod common;
mod db;
mod embedding;
mod mcp;
mod memory;
#[cfg(test)]
mod tests;

use std::sync::Arc;

use actix_web::{App, HttpServer, web};
use crate::db::surreal::Database;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mode = RunMode::from_args(std::env::args().skip(1));

    match mode {
        RunMode::Http => run_http().await,
        RunMode::Mcp => run_mcp().await,
    }
}

async fn run_http() -> std::io::Result<()> {
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

async fn run_mcp() -> std::io::Result<()> {
    let database = Arc::new(Database::new().await.map_err(std::io::Error::other)?);
    mcp::serve(database).await.map_err(std::io::Error::other)
}

enum RunMode {
    Http,
    Mcp,
}

impl RunMode {
    fn from_args(args: impl IntoIterator<Item = String>) -> Self {
        for argument in args {
            if argument == "--mcp" {
                return Self::Mcp;
            }
        }

        Self::Http
    }
}
