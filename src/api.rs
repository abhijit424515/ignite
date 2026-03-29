use actix_web::{HttpResponse, delete, get, post, put, web};
use serde::{Deserialize, Serialize};

use crate::common::error::AppError;
use crate::db::surreal::Database;
use crate::memory;
use crate::memory::model::{EdgeData, MemoryId};

#[derive(Deserialize, Serialize)]
pub struct MemoryPayload {
    pub content: String,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_memory)
        .service(list_edges)
        .service(create_memory)
        .service(update_memory)
        .service(delete_memory)
        .service(search_memory)
        .service(add_edge)
        .service(remove_edge);
}

#[get("/memory/{id}/edge")]
async fn list_edges(database: web::Data<Database>, id: web::Path<String>) -> Result<HttpResponse, AppError> {
    let edges = memory::service::list_edges(database.get_ref(), MemoryId(id.into_inner())).await?;
    Ok(HttpResponse::Ok().json(edges))
}

#[post("/memory/{id}/edge/{target_id}")]
async fn add_edge(
    database: web::Data<Database>,
    path: web::Path<(String, String)>,
    payload: web::Json<EdgeData>,
) -> Result<HttpResponse, AppError> {
    let (from_id, to_id) = path.into_inner();
    memory::service::add_edge(database.get_ref(), MemoryId(from_id), MemoryId(to_id), payload.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[delete("/memory/{id}/edge/{target_id}")]
async fn remove_edge(database: web::Data<Database>, path: web::Path<(String, String)>) -> Result<HttpResponse, AppError> {
    let (from_id, to_id) = path.into_inner();
    memory::service::remove_edge(database.get_ref(), MemoryId(from_id), MemoryId(to_id)).await?;
    Ok(HttpResponse::NoContent().finish())
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
