use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb_types::{RecordId, RecordIdKey, SurrealValue};

use crate::common::error::{AppError, AppResult};
use crate::db::surreal::Database;
use crate::memory::model::{Embedding, Memory, MemoryId};

const TABLE: &str = "memory";

#[derive(Debug, Deserialize, SurrealValue)]
struct StoredMemory {
    id: RecordId,
    content: String,
    embedding: Vec<f32>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, SurrealValue)]
struct StoredMemoryBody {
    content: String,
    embedding: Vec<f32>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct StoredEdge {
    out: RecordId,
}

impl From<StoredMemory> for Memory {
    fn from(memory: StoredMemory) -> Self {
        Self {
            id: MemoryId(format_record_id(&memory.id)),
            content: memory.content,
            embedding: Embedding(memory.embedding),
            created_at: memory.created_at,
            updated_at: memory.updated_at,
        }
    }
}

impl From<&Memory> for StoredMemoryBody {
    fn from(memory: &Memory) -> Self {
        Self {
            content: memory.content.clone(),
            embedding: memory.embedding.0.clone(),
            created_at: memory.created_at,
            updated_at: memory.updated_at,
        }
    }
}

pub async fn insert(database: &Database, memory: &Memory) -> AppResult<()> {
    let _: Option<StoredMemory> = database
        .client()
        .create((TABLE, memory.id.key()))
        .content(StoredMemoryBody::from(memory))
        .await?;

    Ok(())
}

pub async fn get(database: &Database, id: &MemoryId) -> AppResult<Memory> {
    let memory: Option<StoredMemory> = database.client().select((TABLE, id.key())).await?;
    memory.map(Into::into).ok_or(AppError::NotFound)
}

pub async fn update(database: &Database, memory: &Memory) -> AppResult<()> {
    let updated: Option<StoredMemory> = database
        .client()
        .update((TABLE, memory.id.key()))
        .content(StoredMemoryBody::from(memory))
        .await?;

    updated.map(|_| ()).ok_or(AppError::NotFound)
}

pub async fn delete(database: &Database, id: &MemoryId) -> AppResult<()> {
    get(database, id).await?;
    let record_id = record_id(id);

    database
        .client()
        .query(
            "BEGIN TRANSACTION; DELETE memory_edge WHERE in = $memory OR out = $memory; DELETE $memory; COMMIT TRANSACTION;",
        )
        .bind(("memory", record_id))
        .await?;

    Ok(())
}

pub async fn search(
    database: &Database,
    embedding: &[f32],
    top_k: usize,
) -> AppResult<Vec<Memory>> {
    let memories: Vec<StoredMemory> = database.client().select(TABLE).await?;
    let mut memories: Vec<Memory> = memories.into_iter().map(Into::into).collect();

    memories.sort_by(|left, right| {
        similarity(&right.embedding.0, embedding).total_cmp(&similarity(&left.embedding.0, embedding))
    });
    memories.truncate(top_k);

    Ok(memories)
}

pub async fn add_edge(database: &Database, from_id: &MemoryId, to_id: &MemoryId) -> AppResult<()> {
    let from_record = record_id(from_id);
    let to_record = record_id(to_id);
    let mut response = database
        .client()
        .query("SELECT VALUE id FROM memory_edge WHERE in = $from AND out = $to LIMIT 1;")
        .bind(("from", from_record.clone()))
        .bind(("to", to_record.clone()))
        .await?;

    let existing: Vec<RecordId> = response.take(0)?;

    if !existing.is_empty() {
        return Ok(());
    }

    database
        .client()
        .query("RELATE $from->memory_edge->$to SET created_at = time::now();")
        .bind(("from", from_record))
        .bind(("to", to_record))
        .await?;

    Ok(())
}

pub async fn remove_edge(database: &Database, from_id: &MemoryId, to_id: &MemoryId) -> AppResult<()> {
    database
        .client()
        .query("DELETE memory_edge WHERE in = $from AND out = $to;")
        .bind(("from", record_id(from_id)))
        .bind(("to", record_id(to_id)))
        .await?;

    Ok(())
}

pub async fn list_edges(database: &Database, id: &MemoryId) -> AppResult<Vec<MemoryId>> {
    let record_id = record_id(id);
    let mut response = database
        .client()
        .query("SELECT out FROM memory_edge WHERE in = $memory ORDER BY out;")
        .bind(("memory", record_id))
        .await?;

    let edges: Vec<StoredEdge> = response.take(0)?;
    Ok(edges
        .into_iter()
        .map(|edge| MemoryId(format_record_id(&edge.out)))
        .collect())
}

fn record_id(id: &MemoryId) -> RecordId {
    RecordId::new(TABLE, id.key().to_string())
}

fn similarity(left: &[f32], right: &[f32]) -> f32 {
    left.iter().zip(right.iter()).map(|(a, b)| a * b).sum()
}

fn format_record_id(id: &RecordId) -> String {
    let key = match &id.key {
        RecordIdKey::String(value) => value.clone(),
        RecordIdKey::Uuid(value) => value.to_string(),
        value => format!("{value:?}"),
    };

    format!("{}:{key}", id.table.as_str())
}
