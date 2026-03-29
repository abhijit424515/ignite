# Ignite

Ignite is a small Actix Web service for storing searchable memories in embedded SurrealDB.

It can also run as a simple MCP server over stdio.

## What it does

- create, read, update, and delete memories
- search memories with a simple embedding-based similarity query
- add and remove directed edges between memories as separate Surreal relation records
- keep everything in memory with SurrealDB `kv-mem`

## Project shape

```text
src/
 ├── main.rs
 ├── memory/
 │    ├── mod.rs
 │    ├── model.rs
 │    ├── service.rs
 │    ├── repository.rs
 │
 ├── embedding/
 │    ├── mod.rs
 │    └── client.rs
 │
 ├── db/
 │    ├── mod.rs
 │    └── surreal.rs
```

## Run

```bash
cargo run
```

The server listens on `127.0.0.1:8080`.

## MCP

```bash
cargo run -- --mcp
```

This starts a stdio MCP server with these tools:

- `create_memory`
- `get_memory`
- `update_memory`
- `delete_memory`
- `search_memory`
- `add_edge`
- `remove_edge`
- `list_edges`

`add_edge` expects `{ "from_id": "...", "to_id": "...", "data": { "content": "...", ... } }`.

## API

- `POST /memory` with `{ "content": "..." }`
- `GET /memory/{id}`
- `GET /memory/{id}/edge`
- `PUT /memory/{id}` with `{ "content": "..." }`
- `DELETE /memory/{id}`
- `GET /search?q=...`
- `POST /memory/{id}/edge/{target_id}` with `{ "content": "...", ... }`; `content` is required and the whole body is stored under `data` on the edge record
- `DELETE /memory/{id}/edge/{target_id}`

## Checks

```bash
cargo check
cargo check --tests
```

Data lives in embedded in-memory SurrealDB, so it resets whenever the process stops.

Edges are stored separately in the `memory_edge` relation table, not inside each memory record.
