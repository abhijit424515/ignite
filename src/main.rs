mod common;
mod db;
mod embedding;
mod memory;

use actix_web::{App, HttpResponse, HttpServer, delete, get, post, put, web};
use serde::Deserialize;

use crate::common::error::AppError;
use crate::db::surreal::Database;
use crate::memory::model::MemoryId;

#[derive(Deserialize)]
struct MemoryPayload {
    content: String,
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
}

#[get("/memory/{id}")]
async fn get_memory(database: web::Data<Database>, id: web::Path<String>) -> Result<HttpResponse, AppError> {
    let memory = memory::service::get_memory(database.get_ref(), MemoryId(id.into_inner())).await?;
    Ok(HttpResponse::Ok().json(memory))
}

#[post("/memory")]
async fn create_memory(
    database: web::Data<Database>,
    payload: web::Json<MemoryPayload>,
) -> Result<HttpResponse, AppError> {
    let memory = memory::service::create_memory(database.get_ref(), payload.into_inner().content).await?;
    Ok(HttpResponse::Created().json(memory))
}

#[put("/memory/{id}")]
async fn update_memory(
    database: web::Data<Database>,
    id: web::Path<String>,
    payload: web::Json<MemoryPayload>,
) -> Result<HttpResponse, AppError> {
    let memory = memory::service::update_memory(
        database.get_ref(),
        MemoryId(id.into_inner()),
        payload.into_inner().content,
    )
    .await?;

    Ok(HttpResponse::Ok().json(memory))
}

#[delete("/memory/{id}")]
async fn delete_memory(database: web::Data<Database>, id: web::Path<String>) -> Result<HttpResponse, AppError> {
    memory::service::delete_memory(database.get_ref(), MemoryId(id.into_inner())).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[get("/search")]
async fn search_memory(
    database: web::Data<Database>,
    query: web::Query<SearchQuery>,
) -> Result<HttpResponse, AppError> {
    let memories = memory::service::search_memory(database.get_ref(), &query.q).await?;
    Ok(HttpResponse::Ok().json(memories))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database = web::Data::new(Database::new().await.map_err(std::io::Error::other)?);

    HttpServer::new(move || {
        App::new()
            .app_data(database.clone())
            .service(get_memory)
            .service(create_memory)
            .service(update_memory)
            .service(delete_memory)
            .service(search_memory)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
