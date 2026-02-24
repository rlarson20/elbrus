use crate::models::{ScryfallCard, ScryfallError};
use elbrus_core::{OracleCard, Printing};
use elbrus_db::repo::card::CardRepository;
use futures::Stream;
use std::path::Path;

pub struct BulkIngestor {
    pub chunk_size: usize, // default 500, batch insert
}

impl Default for BulkIngestor {
    fn default() -> Self {
        Self { chunk_size: 500 }
    }
}

impl BulkIngestor {
    /// Stream-parse a `bulk-data` JSON file, yield batches.
    pub fn ingest_file(
        &self,
        path: &Path,
    ) -> impl Stream<Item = Result<Vec<(OracleCard, Printing)>, ScryfallError>> {
        let (tx, rx) = tokio::sync::mpsc::channel(2);
        let chunk_size = self.chunk_size;
        let path_buf = path.to_path_buf();

        tokio::task::spawn_blocking(move || {
            let file = match std::fs::File::open(&path_buf) {
                Ok(f) => f,
                Err(e) => {
                    let _ = tx.blocking_send(Err(ScryfallError::Io(e)));
                    return;
                }
            };

            let reader = std::io::BufReader::new(file);
            use std::io::Read;

            let mut batch = Vec::with_capacity(chunk_size);
            let mut buf = Vec::new();
            let mut depth = 0;
            let mut in_string = false;
            let mut escape = false;

            for byte_res in reader.bytes() {
                let Ok(b) = byte_res else { break };

                if depth > 0 {
                    buf.push(b);
                }

                if !in_string {
                    if b == b'{' {
                        if depth == 0 {
                            buf.push(b);
                        }
                        depth += 1;
                    } else if b == b'}' {
                        depth -= 1;
                        if depth == 0 {
                            // We have a complete object
                            match serde_json::from_slice::<ScryfallCard>(&buf) {
                                Ok(card) => {
                                    batch.push(card.into_core());
                                    if batch.len() == chunk_size {
                                        let to_send = std::mem::replace(
                                            &mut batch,
                                            Vec::with_capacity(chunk_size),
                                        );
                                        if tx.blocking_send(Ok(to_send)).is_err() {
                                            return;
                                        }
                                    }
                                }
                                Err(e) => {
                                    let _ = tx.blocking_send(Err(ScryfallError::Json(e)));
                                    return;
                                }
                            }
                            buf.clear();
                        }
                    } else if b == b'"' {
                        in_string = true;
                    }
                } else if escape {
                    escape = false;
                } else if b == b'\\' {
                    escape = true;
                } else if b == b'"' {
                    in_string = false;
                }
            }

            if !batch.is_empty() {
                let _ = tx.blocking_send(Ok(batch));
            }
        });

        futures::stream::unfold(rx, |mut rx| async move {
            rx.recv().await.map(|item| (item, rx))
        })
    }

    /// Convenience: ingest directly into db.
    pub async fn ingest_into_db(
        &self,
        path: &Path,
        db: &dyn CardRepository,
    ) -> Result<IngestStats, ScryfallError> {
        use futures::StreamExt;

        let mut stats = IngestStats {
            cards_processed: 0,
            cards_inserted: 0,
            cards_updated: 0,
            duration: std::time::Duration::from_secs(0),
        };
        let start = std::time::Instant::now();

        // Wait, traits from other crates need to be available? StreamExt is imported.
        let mut stream = Box::pin(self.ingest_file(path));

        while let Some(res) = stream.next().await {
            let batch = res?;
            for (oracle_card, printing) in batch {
                db.upsert_oracle(&oracle_card)
                    .await
                    .map_err(|e| ScryfallError::Network(format!("DB error: {e:?}")))?;
                db.upsert_printing(&printing)
                    .await
                    .map_err(|e| ScryfallError::Network(format!("DB error: {e:?}")))?;
                stats.cards_processed += 1;
                stats.cards_inserted += 1;
            }
        }

        stats.duration = start.elapsed();
        Ok(stats)
    }
}

pub struct IngestStats {
    pub cards_processed: u64,
    pub cards_inserted: u64,
    pub cards_updated: u64,
    pub duration: std::time::Duration,
}

pub use std::time::Duration;
