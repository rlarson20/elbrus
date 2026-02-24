use crate::backend::{DbError, Row, StorageBackend, Transaction};
use futures::future::BoxFuture;
use serde_json::Value;
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct SqliteBackend {
    pub pool: SqlitePool,
}

impl SqliteBackend {
    pub async fn open(path: &str) -> Result<Self, DbError> {
        let options = SqliteConnectOptions::from_str(path)
            .map_err(|e| DbError::Unknown(e.to_string()))?
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .connect_with(options)
            .await
            .map_err(|e| DbError::Sql(e.to_string()))?;

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| DbError::Sql(e.to_string()))?;

        Ok(Self { pool })
    }

    pub async fn open_in_memory() -> Result<Self, DbError> {
        Self::open("sqlite::memory:").await
    }
}

pub struct SqliteTransaction<'c> {
    pub tx: sqlx::Transaction<'c, sqlx::Sqlite>,
}

impl<'c> Transaction for SqliteTransaction<'c> {}

#[async_trait::async_trait]
impl StorageBackend for SqliteBackend {
    async fn execute(&self, sql: &str, params: &[Value]) -> Result<u64, DbError> {
        let mut q = sqlx::query(sql);
        for param in params {
            q = match param {
                Value::Null => q.bind(None::<String>),
                Value::Bool(b) => q.bind(b),
                Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        q.bind(i)
                    } else if let Some(f) = n.as_f64() {
                        q.bind(f)
                    } else {
                        q.bind(n.to_string())
                    }
                }
                Value::String(s) => q.bind(s),
                val => q.bind(val.to_string()),
            };
        }

        let result = q
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Sql(e.to_string()))?;

        Ok(result.rows_affected())
    }

    async fn query(&self, sql: &str, params: &[Value]) -> Result<Vec<Row>, DbError> {
        let mut q = sqlx::query(sql);
        for param in params {
            q = match param {
                Value::Null => q.bind(None::<String>),
                Value::Bool(b) => q.bind(b),
                Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        q.bind(i)
                    } else if let Some(f) = n.as_f64() {
                        q.bind(f)
                    } else {
                        q.bind(n.to_string())
                    }
                }
                Value::String(s) => q.bind(s),
                val => q.bind(val.to_string()),
            };
        }

        let rows = q
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Sql(e.to_string()))?;

        Ok(rows.into_iter().map(|_| Row).collect())
    }

    async fn transaction<F, T>(&self, f: F) -> Result<T, DbError>
    where
        T: Send,
        F: FnOnce(&mut dyn Transaction) -> BoxFuture<'_, Result<T, DbError>> + Send,
    {
        let tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DbError::Sql(e.to_string()))?;
        let mut wrapper = SqliteTransaction { tx };

        // This expects the closure to take mutable reference to dyn Transaction
        let result = f(&mut wrapper).await?;
        wrapper
            .tx
            .commit()
            .await
            .map_err(|e| DbError::Sql(e.to_string()))?;
        Ok(result)
    }
}
