# Ignite

# 🧠 1. Minimal Rust Project Structure

Keep it flat and boring:

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

---

## 🧩 What each layer does (strictly)

### `model.rs` → types only

```rust
pub struct Memory {
    pub id: MemoryId,
    pub content: String,
    pub embedding: Embedding,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct MemoryId(pub String);
pub struct Embedding(pub Vec<f32>);
```

👉 No logic here. Just types.

---

### `embedding/client.rs` → external call only

```rust
pub async fn embed(text: &str) -> Result<Vec<f32>> {
    // call Google Embedding API
}
```

👉 Single responsibility: get embeddings

---

### `repository.rs` → DB only

```rust
pub async fn insert(memory: &Memory) -> Result<()> { ... }

pub async fn get(id: &MemoryId) -> Result<Memory> { ... }

pub async fn update(memory: &Memory) -> Result<()> { ... }

pub async fn delete(id: &MemoryId) -> Result<()> { ... }

pub async fn search(embedding: &[f32], top_k: usize) -> Result<Vec<Memory>> { ... }
```

👉 No business logic, no embedding logic

---

### `service.rs` → orchestration (THIS is your brain)

```rust
pub async fn create_memory(content: String) -> Result<Memory> {
    let embedding = embed(&content).await?;
    
    let memory = Memory {
        id: MemoryId(generate_id()),
        content,
        embedding: Embedding(embedding),
        created_at: now(),
        updated_at: now(),
    };

    repository::insert(&memory).await?;
    Ok(memory)
}
```

---

### Update flow (important)

```rust
pub async fn update_memory(id: MemoryId, content: String) -> Result<()> {
    let embedding = embed(&content).await?;

    let mut memory = repository::get(&id).await?;
    memory.content = content;
    memory.embedding = Embedding(embedding);
    memory.updated_at = now();

    repository::update(&memory).await
}
```

---

### Search flow (clean)

```rust
pub async fn search_memory(query: &str) -> Result<Vec<Memory>> {
    let embedding = embed(query).await?;
    repository::search(&embedding, 5).await
}
```

---

# 🧠 2. SurrealDB Schema (minimal)

You don’t need much.

---

## 📦 Table definition

```sql
DEFINE TABLE memory SCHEMALESS;
```

---

## 🧱 Fields (optional but good)

```sql
DEFINE FIELD content ON memory TYPE string;
DEFINE FIELD embedding ON memory TYPE array;
DEFINE FIELD created_at ON memory TYPE datetime;
DEFINE FIELD updated_at ON memory TYPE datetime;
```

---

## 🔍 Vector index (important)

```sql
DEFINE INDEX memory_embedding_idx
ON memory
FIELDS embedding
SEARCH ANALYZER vector;
```

👉 (Exact syntax may evolve — but concept = vector index)

---

# 🧠 3. Mapping Rust → SurrealDB

## ➕ Insert

```rust
let query = r#"
CREATE memory SET
    id = $id,
    content = $content,
    embedding = $embedding,
    created_at = time::now(),
    updated_at = time::now()
"#;
```

---

## 🔍 Search (core idea)

```rust
let query = r#"
SELECT *, vector::similarity(embedding, $query_embedding) AS score
FROM memory
ORDER BY score DESC
LIMIT $limit
"#;
```

👉 This is your entire retrieval system

---

## ✏️ Update

```rust
let query = r#"
UPDATE memory:$id SET
    content = $content,
    embedding = $embedding,
    updated_at = time::now()
"#;
```

---

## ❌ Delete

```rust
let query = r#"
DELETE memory:$id
"#;
```

---

## 📥 Get

```rust
let query = r#"
SELECT * FROM memory:$id
"#;
```

---

# 🧠 4. Important Implementation Details

## 🧩 ID generation

Keep it simple:

```rust
fn generate_id() -> String {
    format!("memory:{}", uuid::Uuid::new_v4())
}
```

---

## 🧠 Time

```rust
fn now() -> DateTime<Utc> {
    Utc::now()
}
```

---

## ⚠️ Embedding consistency

Hardcode model name somewhere:

```rust
const EMBEDDING_MODEL: &str = "google-embedding-2";
```

---

# 🧠 5. API Layer (very thin)

You don’t need a framework yet, but conceptually:

```rust
POST /memory
GET /search?q=...
PUT /memory/:id
DELETE /memory/:id
```

---

# 🧠 6. How Raycast / MCP plug in

They both just call:

* `create_memory`
* `search_memory`
* `update_memory`
* `delete_memory`

👉 No special logic needed

---

# 🧠 Final Mental Model

```text
Rust Service Layer (your logic)
        ↓
SurrealDB (storage + search)
        ↓
Embedding API (Google)
```

---

# 🧠 What you now have

With just this:

* you can store memories
* you can search semantically
* you can edit/delete
* it’s type-safe
* it’s minimal

---

# 🧠 What NOT to add yet

* graph edges
* ranking logic
* caching
* batching
* background jobs
