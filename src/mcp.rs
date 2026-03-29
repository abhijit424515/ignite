use std::sync::Arc;
use std::time::Duration;

use fastmcp::tool::ToolFn;
use fastmcp::{Error as McpError, McpServerBuilder, Result as McpResult, TransportConfig};
use serde::Deserialize;
use serde::Serialize;
use serde_json::{Value, json};

use crate::common::error::AppError;
use crate::db::surreal::Database;
use crate::memory::model::{EdgeData, MemoryId};
use crate::memory::service;

pub async fn serve(database: Arc<Database>) -> McpResult<()> {
    let mut server = McpServerBuilder::new()
        .with_transport(TransportConfig::Stdio)
        .with_default_timeout(Duration::from_secs(30))
        .with_tool(create_memory_tool(database.clone()))
        .with_tool(get_memory_tool(database.clone()))
        .with_tool(update_memory_tool(database.clone()))
        .with_tool(delete_memory_tool(database.clone()))
        .with_tool(search_memory_tool(database.clone()))
        .with_tool(add_edge_tool(database.clone()))
        .with_tool(remove_edge_tool(database.clone()))
        .with_tool(list_edges_tool(database))
        .build()?;

    server.start().await
}

fn create_memory_tool(database: Arc<Database>) -> impl fastmcp::Tool {
    ToolFn::new(
        "create_memory",
        "Create a memory node from content.",
        json!({
            "type": "object",
            "properties": {
                "content": { "type": "string" }
            },
            "required": ["content"],
            "additionalProperties": false
        }),
        move |params, _context| {
            let database = database.clone();
            async move {
                let params: MemoryContentParams = parse_params(params)?;
                let memory = service::create_memory(database.as_ref(), params.content)
                    .await
                    .map_err(map_app_error)?;
                to_json(memory)
            }
        },
    )
}

fn get_memory_tool(database: Arc<Database>) -> impl fastmcp::Tool {
    ToolFn::new(
        "get_memory",
        "Load a memory node by id.",
        id_schema(),
        move |params, _context| {
            let database = database.clone();
            async move {
                let params: IdParams = parse_params(params)?;
                let memory = service::get_memory(database.as_ref(), MemoryId(params.id))
                    .await
                    .map_err(map_app_error)?;
                to_json(memory)
            }
        },
    )
}

fn update_memory_tool(database: Arc<Database>) -> impl fastmcp::Tool {
    ToolFn::new(
        "update_memory",
        "Update a memory node and refresh its embedding.",
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string" },
                "content": { "type": "string" }
            },
            "required": ["id", "content"],
            "additionalProperties": false
        }),
        move |params, _context| {
            let database = database.clone();
            async move {
                let params: UpdateMemoryParams = parse_params(params)?;
                let memory =
                    service::update_memory(database.as_ref(), MemoryId(params.id), params.content)
                        .await
                        .map_err(map_app_error)?;
                to_json(memory)
            }
        },
    )
}

fn delete_memory_tool(database: Arc<Database>) -> impl fastmcp::Tool {
    ToolFn::new(
        "delete_memory",
        "Delete a memory node and its attached edges.",
        id_schema(),
        move |params, _context| {
            let database = database.clone();
            async move {
                let params: IdParams = parse_params(params)?;
                service::delete_memory(database.as_ref(), MemoryId(params.id))
                    .await
                    .map_err(map_app_error)?;
                Ok(json!({ "deleted": true }))
            }
        },
    )
}

fn search_memory_tool(database: Arc<Database>) -> impl fastmcp::Tool {
    ToolFn::new(
        "search_memory",
        "Search memories by semantic similarity.",
        json!({
            "type": "object",
            "properties": {
                "query": { "type": "string" }
            },
            "required": ["query"],
            "additionalProperties": false
        }),
        move |params, _context| {
            let database = database.clone();
            async move {
                let params: SearchParams = parse_params(params)?;
                let memories = service::search_memory(database.as_ref(), &params.query)
                    .await
                    .map_err(map_app_error)?;
                to_json(memories)
            }
        },
    )
}

fn add_edge_tool(database: Arc<Database>) -> impl fastmcp::Tool {
    ToolFn::new(
        "add_edge",
        "Create or replace a directed edge between two memories.",
        json!({
            "type": "object",
            "properties": {
                "from_id": { "type": "string" },
                "to_id": { "type": "string" },
                "data": {
                    "type": "object",
                    "properties": {
                        "content": { "type": "string" }
                    },
                    "required": ["content"],
                    "additionalProperties": true
                }
            },
            "required": ["from_id", "to_id", "data"],
            "additionalProperties": false
        }),
        move |params, _context| {
            let database = database.clone();
            async move {
                let params: AddEdgeParams = parse_params(params)?;
                service::add_edge(
                    database.as_ref(),
                    MemoryId(params.from_id),
                    MemoryId(params.to_id),
                    params.data,
                )
                .await
                .map_err(map_app_error)?;
                Ok(json!({ "ok": true }))
            }
        },
    )
}

fn remove_edge_tool(database: Arc<Database>) -> impl fastmcp::Tool {
    ToolFn::new(
        "remove_edge",
        "Remove a directed edge between two memories.",
        json!({
            "type": "object",
            "properties": {
                "from_id": { "type": "string" },
                "to_id": { "type": "string" }
            },
            "required": ["from_id", "to_id"],
            "additionalProperties": false
        }),
        move |params, _context| {
            let database = database.clone();
            async move {
                let params: EdgeIdsParams = parse_params(params)?;
                service::remove_edge(
                    database.as_ref(),
                    MemoryId(params.from_id),
                    MemoryId(params.to_id),
                )
                .await
                .map_err(map_app_error)?;
                Ok(json!({ "removed": true }))
            }
        },
    )
}

fn list_edges_tool(database: Arc<Database>) -> impl fastmcp::Tool {
    ToolFn::new(
        "list_edges",
        "List outgoing edge targets for a memory node.",
        id_schema(),
        move |params, _context| {
            let database = database.clone();
            async move {
                let params: IdParams = parse_params(params)?;
                let edges = service::list_edges(database.as_ref(), MemoryId(params.id))
                    .await
                    .map_err(map_app_error)?;
                to_json(edges)
            }
        },
    )
}

fn id_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "id": { "type": "string" }
        },
        "required": ["id"],
        "additionalProperties": false
    })
}

fn parse_params<T>(params: Value) -> McpResult<T>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_value(params).map_err(|error| McpError::InvalidInput(error.to_string()))
}

fn to_json<T>(value: T) -> McpResult<Value>
where
    T: Serialize,
{
    serde_json::to_value(value).map_err(McpError::from)
}

fn map_app_error(error: AppError) -> McpError {
    match error {
        AppError::BadRequest(message) => McpError::InvalidInput(message),
        AppError::NotFound => McpError::InvalidInput("memory not found".to_string()),
        AppError::Internal => McpError::ToolExecution("internal server error".to_string()),
    }
}

#[derive(Deserialize)]
struct MemoryContentParams {
    content: String,
}

#[derive(Deserialize)]
struct IdParams {
    id: String,
}

#[derive(Deserialize)]
struct UpdateMemoryParams {
    id: String,
    content: String,
}

#[derive(Deserialize)]
struct SearchParams {
    query: String,
}

#[derive(Deserialize)]
struct EdgeIdsParams {
    from_id: String,
    to_id: String,
}

#[derive(Deserialize)]
struct AddEdgeParams {
    from_id: String,
    to_id: String,
    data: EdgeData,
}
