use crate::models::ScryfallError;
use elbrus_core::{OracleCard, Printing};
use elbrus_db::repo::card::CardRepository;
use futures::Stream;
use std::path::Path;

pub struct BulkIngestor {
    pub chunk_size: usize, // default 500, batch insert
}

impl BulkIngestor {
    /// Stream-parse a `bulk-data` JSON file, yield batches.
    pub fn ingest_file(
        &self,
        _path: &Path,
    ) -> impl Stream<Item = Result<Vec<(OracleCard, Printing)>, ScryfallError>> {
        futures::stream::empty()
    }

    /// Convenience: ingest directly into db.
    pub async fn ingest_into_db(
        &self,
        _path: &Path,
        _db: &dyn CardRepository,
    ) -> Result<IngestStats, ScryfallError> {
        todo!()
    }
}

pub struct IngestStats {
    pub cards_processed: u64,
    pub cards_inserted: u64,
    pub cards_updated: u64,
    pub duration: std::time::Duration,
}

pub use std::time::Duration;
