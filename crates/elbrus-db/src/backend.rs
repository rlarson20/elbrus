use futures::future::BoxFuture;
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("SQL error: {0}")]
    Sql(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub struct Row;

#[async_trait::async_trait]
pub trait StorageBackend: Send + Sync {
    async fn execute(&self, sql: &str, params: &[Value]) -> Result<u64, DbError>;
    async fn query(&self, sql: &str, params: &[Value]) -> Result<Vec<Row>, DbError>;
    async fn transaction<F, T>(&self, f: F) -> Result<T, DbError>
    where
        T: Send,
        F: FnOnce(&mut dyn Transaction) -> BoxFuture<'_, Result<T, DbError>> + Send;
}

pub trait Transaction: Send {}
