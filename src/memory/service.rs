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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::error::AppError;
    use crate::db::surreal::Database;

    #[actix_web::test]
    async fn memory_flow_works() {
        let database = Database::new().await.expect("database should initialize");

        let first = create_memory(&database, "rust web memory".to_string())
            .await
            .expect("memory should be created");
        let second = create_memory(&database, "surreal graph node".to_string())
            .await
            .expect("memory should be created");

        let loaded = get_memory(&database, first.id.clone())
            .await
            .expect("memory should load");
        assert_eq!(loaded.content, "rust web memory");

        let updated = update_memory(&database, first.id.clone(), "rust actix memory".to_string())
            .await
            .expect("memory should update");
        assert_eq!(updated.content, "rust actix memory");

        let search_results = search_memory(&database, "actix")
            .await
            .expect("search should work");
        assert!(search_results.iter().any(|memory| memory.id == first.id));

        add_edge(&database, first.id.clone(), second.id.clone())
            .await
            .expect("edge should be added");
        let edges = list_edges(&database, first.id.clone())
            .await
            .expect("edges should load");
        assert_eq!(edges, vec![second.id.clone()]);

        remove_edge(&database, first.id.clone(), second.id.clone())
            .await
            .expect("edge should be removed");
        let edges = list_edges(&database, first.id.clone())
            .await
            .expect("edges should load");
        assert!(edges.is_empty());

        delete_memory(&database, second.id.clone())
            .await
            .expect("memory should delete");

        let error = get_memory(&database, second.id.clone())
            .await
            .expect_err("deleted memory should be missing");
        assert!(matches!(error, AppError::NotFound));
    }

    #[actix_web::test]
    async fn add_edge_rejects_self_links() {
        let database = Database::new().await.expect("database should initialize");
        let memory = create_memory(&database, "self edge check".to_string())
            .await
            .expect("memory should be created");

        let error = add_edge(&database, memory.id.clone(), memory.id.clone())
            .await
            .expect_err("self edge should fail");

        assert!(matches!(error, AppError::BadRequest(_)));
    }
}
