use elbrus_db::repo::card::CardRepository;
use elbrus_db::sqlite::SqliteBackend;
use elbrus_scryfall::bulk::BulkIngestor;
use std::path::PathBuf;

#[tokio::test]
async fn test_ingest_sample() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut sample_path = PathBuf::from(manifest_dir);
    sample_path.push("tests/data/sample.json");

    let db = SqliteBackend::open_in_memory()
        .await
        .expect("Failed to open DB");
    let ingestor = BulkIngestor { chunk_size: 10 };

    let stats = ingestor
        .ingest_into_db(&sample_path, &db)
        .await
        .expect("Failed to ingest");

    // We expect exactly 50 cards evaluated in sample.json (generated with seed 42)
    assert_eq!(stats.cards_processed, 50);

    // Validate card existence of the first card from the sample
    let oracle_id = uuid::Uuid::parse_str("3397aa3d-bf73-4ca3-a806-059361603079").unwrap();
    let oracle_card = db
        .get_oracle(oracle_id)
        .await
        .unwrap()
        .expect("Oracle card missing");

    assert_eq!(oracle_card.oracle_id, oracle_id);
    assert_eq!(oracle_card.faces.len(), 1);
    assert_eq!(oracle_card.faces[0].name.as_ref(), "Test of Faith");

    let printing_id = uuid::Uuid::parse_str("67ba07ca-7be4-400e-a104-f7bbd527b6b4").unwrap();
    let printing = db
        .get_by_id(printing_id)
        .await
        .unwrap()
        .expect("Printing missing");

    assert_eq!(printing.id, printing_id);
    assert_eq!(printing.set_code.as_ref(), "mma");
    assert_eq!(printing.collector_number.as_ref(), "33");
}
