use crate::backend::DbError;
use elbrus_core::{Format, OracleCard, Printing};
use uuid::Uuid;

#[async_trait::async_trait]
pub trait CardRepository: Send + Sync {
    async fn upsert_oracle(&self, card: &OracleCard) -> Result<(), DbError>;
    async fn upsert_printing(&self, p: &Printing) -> Result<(), DbError>;
    async fn get_by_id(&self, id: Uuid) -> Result<Option<Printing>, DbError>;
    async fn get_oracle(&self, oracle_id: Uuid) -> Result<Option<OracleCard>, DbError>;
    async fn search_name(&self, q: &str, limit: u32) -> Result<Vec<OracleCard>, DbError>;
    async fn search_fts(&self, q: &str, limit: u32) -> Result<Vec<OracleCard>, DbError>;
    async fn cards_in_set(&self, set_code: &str) -> Result<Vec<Printing>, DbError>;
    async fn legal_in_format(&self, format: &Format) -> Result<Vec<OracleCard>, DbError>;
}
