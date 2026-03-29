# Ignite

Ignite is a small Actix Web service for storing searchable memories in embedded SurrealDB.

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

## API

- `POST /memory` with `{ "content": "..." }`
- `GET /memory/{id}`
- `GET /memory/{id}/edge`
- `PUT /memory/{id}` with `{ "content": "..." }`
- `DELETE /memory/{id}`
- `GET /search?q=...`
- `POST /memory/{id}/edge/{target_id}`
- `DELETE /memory/{id}/edge/{target_id}`

## Checks

```bash
cargo check
cargo test
```

Data lives in embedded in-memory SurrealDB, so it resets whenever the process stops.

Edges are stored separately in the `memory_edge` relation table, not inside each memory record.
