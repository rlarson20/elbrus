use chrono::Utc;
use elbrus_core::{
    CardFace, CardLayout, CollectionEntry, Condition, OracleCard, PriceSnapshot, Printing, Rarity,
    color::ColorSet, legality::Legalities, oracle::OracleText, types::TypeLine,
};
use elbrus_db::repo::{CardRepository, CollectionRepository, PriceRepository};
use elbrus_db::sqlite::SqliteBackend;
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

#[tokio::test]
async fn test_db_integration() {
    let db = SqliteBackend::open_in_memory().await.unwrap();

    // 1. Ingest test data
    let oracle_id = Uuid::new_v4();
    let printing_id = Uuid::new_v4();

    let card = OracleCard {
        oracle_id,
        layout: CardLayout::Normal,
        faces: smallvec::smallvec![CardFace {
            name: "Test Card".into(),
            mana_cost: None,
            type_line: TypeLine::default(),
            oracle_text: OracleText::default(),
            colors: ColorSet::empty(),
            power: None,
            toughness: None,
            loyalty: None,
            defense: None,
            flavor_text: None,
        }],
        color_identity: ColorSet::empty(),
        keywords: vec![],
        legalities: Legalities::default(),
        edh_rank: Some(100),
        reserved: false,
    };

    db.upsert_oracle(&card).await.unwrap();

    let printing = Printing {
        id: printing_id,
        oracle_id,
        set_code: "TST".into(),
        collector_number: "1".into(),
        rarity: Rarity::Common,
        lang: "en".into(),
        released_at: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        image_uris: None,
        promo: false,
        digital: false,
        full_art: false,
        textless: false,
        reprint: false,
        prices: None,
    };

    db.upsert_printing(&printing).await.unwrap();

    // Verify card is retrievable
    let retrieved = db.get_oracle(oracle_id).await.unwrap().unwrap();
    assert_eq!(retrieved.oracle_id, oracle_id);
    assert_eq!(retrieved.faces[0].name.as_ref(), "Test Card");

    let retrieved_print = db.get_by_id(printing_id).await.unwrap().unwrap();
    assert_eq!(retrieved_print.id, printing_id);

    // 2. Price snapshot query
    let snapshot = PriceSnapshot {
        usd: Some(Decimal::from_str("1.50").unwrap()),
        usd_foil: Some(Decimal::from_str("3.00").unwrap()),
        eur: None,
        tix: None,
        fetched_at: Utc::now(),
    };

    db.insert_snapshot(printing_id, &snapshot).await.unwrap();

    // Query latest price
    let latest_price = db.get_latest_price(printing_id).await.unwrap().unwrap();
    assert_eq!(latest_price.usd, snapshot.usd);

    // Query price history
    let history = db.get_price_history(printing_id).await.unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].usd_foil, snapshot.usd_foil);

    // 3. Collection CRUD
    // Create collection
    let coll = db
        .create_collection("My Deck", Some("A test deck"))
        .await
        .unwrap();
    assert_eq!(coll.name.as_ref(), "My Deck");

    let entry = CollectionEntry {
        collection_id: coll.id,
        printing_id,
        quantity: 4,
        condition: Condition::NearMint,
        foil: false,
        notes: Some("Playset".into()),
    };

    db.upsert_card(&entry).await.unwrap();

    let mut fetched_coll = db.get_collection(coll.id).await.unwrap().unwrap();
    assert_eq!(fetched_coll.entries.len(), 1);
    assert_eq!(fetched_coll.entries[0].quantity, 4);

    let coll_value = db.get_collection_value(coll.id).await.unwrap();
    assert_eq!(coll_value, Decimal::from_str("6.00").unwrap()); // 4 * 1.50 (not foil)

    let entry_foil = CollectionEntry {
        collection_id: coll.id,
        printing_id,
        quantity: 2,
        condition: Condition::NearMint,
        foil: true,
        notes: None,
    };

    db.upsert_card(&entry_foil).await.unwrap();

    let coll_value2 = db.get_collection_value(coll.id).await.unwrap();
    // 4 * 1.50 + 2 * 3.00 = 6.00 + 6.00 = 12.00
    assert_eq!(coll_value2, Decimal::from_str("12.00").unwrap());

    // List collections
    let collist = db.list_collections().await.unwrap();
    assert_eq!(collist.len(), 1);

    // Remove card
    db.remove_card(coll.id, printing_id, Condition::NearMint, false)
        .await
        .unwrap();
    fetched_coll = db.get_collection(coll.id).await.unwrap().unwrap();
    assert_eq!(fetched_coll.entries.len(), 1); // foil is still there

    // Delete collection
    db.delete_collection(coll.id).await.unwrap();
    let collist_after = db.list_collections().await.unwrap();
    assert_eq!(collist_after.len(), 0);
}
