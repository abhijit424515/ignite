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
        client.query("DEFINE TABLE memory SCHEMALESS;").await?;

        Ok(Self { client })
    }

    pub fn client(&self) -> &Surreal<Db> {
        &self.client
    }
}
