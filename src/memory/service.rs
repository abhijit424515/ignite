use chrono::Utc;
use uuid::Uuid;

use crate::common::error::AppResult;
use crate::db::surreal::Database;
use crate::embedding::client::embed;
use crate::memory::model::{Embedding, Memory, MemoryId};
use crate::memory::repository;

pub async fn get_memory(database: &Database, id: MemoryId) -> AppResult<Memory> {
    repository::get(database, &id).await
}

pub async fn create_memory(database: &Database, content: String) -> AppResult<Memory> {
    let embedding = embed(&content).await?;
    let timestamp = Utc::now();

    let memory = Memory {
        id: MemoryId(generate_id()),
        content,
        embedding: Embedding(embedding),
        created_at: timestamp,
        updated_at: timestamp,
    };

    repository::insert(database, &memory).await?;
    Ok(memory)
}

pub async fn update_memory(database: &Database, id: MemoryId, content: String) -> AppResult<Memory> {
    let embedding = embed(&content).await?;

    let mut memory = repository::get(database, &id).await?;
    memory.content = content;
    memory.embedding = Embedding(embedding);
    memory.updated_at = Utc::now();

    repository::update(database, &memory).await?;
    Ok(memory)
}

pub async fn delete_memory(database: &Database, id: MemoryId) -> AppResult<()> {
    repository::delete(database, &id).await
}

pub async fn add_edge(database: &Database, from_id: MemoryId, to_id: MemoryId) -> AppResult<()> {
    if from_id == to_id {
        return Err(crate::common::error::AppError::BadRequest(
            "cannot create an edge to the same memory".to_string(),
        ));
    }

    repository::get(database, &to_id).await?;
    repository::add_edge(database, &from_id, &to_id).await
}

pub async fn remove_edge(database: &Database, from_id: MemoryId, to_id: MemoryId) -> AppResult<()> {
    repository::get(database, &from_id).await?;
    repository::get(database, &to_id).await?;
    repository::remove_edge(database, &from_id, &to_id).await
}

pub async fn list_edges(database: &Database, id: MemoryId) -> AppResult<Vec<MemoryId>> {
    repository::get(database, &id).await?;
    repository::list_edges(database, &id).await
}

pub async fn search_memory(database: &Database, query: &str) -> AppResult<Vec<Memory>> {
    let embedding = embed(query).await?;
    repository::search(database, &embedding, 5).await
}

fn generate_id() -> String {
    format!("memory:{}", Uuid::new_v4())
}
