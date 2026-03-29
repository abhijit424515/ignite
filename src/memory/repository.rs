use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb_types::{RecordId, RecordIdKey, SurrealValue};

use crate::common::error::{AppError, AppResult};
use crate::db::surreal::Database;
use crate::memory::model::{Embedding, Memory, MemoryId};

const TABLE: &str = "memory";

#[derive(Debug, Serialize, SurrealValue)]
struct MemoryContent {
    content: String,
    embedding: Vec<f32>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DbMemory {
    id: RecordId,
    content: String,
    embedding: Vec<f32>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<&Memory> for MemoryContent {
    fn from(memory: &Memory) -> Self {
        Self {
            content: memory.content.clone(),
            embedding: memory.embedding.0.clone(),
            created_at: memory.created_at,
            updated_at: memory.updated_at,
        }
    }
}

impl From<DbMemory> for Memory {
    fn from(memory: DbMemory) -> Self {
        Self {
            id: MemoryId(format_record_id(&memory.id)),
            content: memory.content,
            embedding: Embedding(memory.embedding),
            created_at: memory.created_at,
            updated_at: memory.updated_at,
        }
    }
}

pub async fn insert(database: &Database, memory: &Memory) -> AppResult<()> {
    let _: Option<DbMemory> = database
        .client()
        .create((TABLE, memory.id.key()))
        .content(MemoryContent::from(memory))
        .await?;

    Ok(())
}

pub async fn get(database: &Database, id: &MemoryId) -> AppResult<Memory> {
    let memory: Option<DbMemory> = database.client().select((TABLE, id.key())).await?;
    memory.map(Into::into).ok_or(AppError::NotFound)
}

pub async fn update(database: &Database, memory: &Memory) -> AppResult<()> {
    let updated: Option<DbMemory> = database
        .client()
        .update((TABLE, memory.id.key()))
        .content(MemoryContent::from(memory))
        .await?;

    updated.map(|_| ()).ok_or(AppError::NotFound)
}

pub async fn delete(database: &Database, id: &MemoryId) -> AppResult<()> {
    let deleted: Option<DbMemory> = database.client().delete((TABLE, id.key())).await?;
    deleted.map(|_| ()).ok_or(AppError::NotFound)
}

pub async fn search(
    database: &Database,
    embedding: &[f32],
    top_k: usize,
) -> AppResult<Vec<Memory>> {
    let mut response = database
        .client()
        .query(
            "SELECT * FROM memory WHERE embedding <|$limit,COSINE|> $embedding ORDER BY vector::distance::knn();",
        )
        .bind(("embedding", embedding.to_vec()))
        .bind(("limit", top_k))
        .await?;

    let memories: Vec<DbMemory> = response.take(0)?;
    Ok(memories.into_iter().map(Into::into).collect())
}

fn format_record_id(id: &RecordId) -> String {
    let key = match &id.key {
        RecordIdKey::String(value) => value.clone(),
        RecordIdKey::Uuid(value) => value.to_string(),
        value => format!("{value:?}"),
    };

    format!("{}:{key}", id.table.as_str())
}
