use surrealdb::Surreal;
use surrealdb::engine::local::{Db, Mem};

use crate::common::error::AppResult;

pub struct Database {
    client: Surreal<Db>,
}

impl Database {
    pub async fn new() -> AppResult<Self> {
        let client = Surreal::new::<Mem>(()).await?;
        client.use_ns("ignite").use_db("ignite").await?;
        client
            .query(
                "DEFINE TABLE memory SCHEMALESS; DEFINE FIELD content ON memory TYPE string; DEFINE FIELD embedding ON memory TYPE array; DEFINE FIELD created_at ON memory TYPE datetime; DEFINE FIELD updated_at ON memory TYPE datetime; DEFINE TABLE memory_edge TYPE RELATION IN memory OUT memory ENFORCED SCHEMALESS; DEFINE FIELD content ON memory_edge.data TYPE string; DEFINE FIELD created_at ON memory_edge TYPE datetime;",
            )
            .await?;

        Ok(Self { client })
    }

    pub fn client(&self) -> &Surreal<Db> {
        &self.client
    }
}
