# Phase 1 — Foundation (`elbrus-db` → `elbrus-scryfall` → `elbrus-core`)

> **Build order rationale:** get data moving before implementing parsers. `ManaCost::parse`,
> `TypeLine::parse`, etc. are implemented _after_ ingest works — at that point you have a real
> corpus to write tests against. Use fallback/stub conversions in `convert.rs` until then (see 1B).

## 1A. `elbrus-db` — SQLite Schema & Implementation

- [x] Create `crates/elbrus-db/migrations/` directory
- [x] Write migration `001_initial_schema.sql`
  - [x] `oracle_cards` table: `oracle_id UUID PK`, `layout TEXT`, `color_identity INTEGER`, `keywords TEXT (JSON)`, `legalities TEXT (JSON)`, `edh_rank INTEGER`, `reserved BOOLEAN`
  - [x] `card_faces` table: `oracle_id UUID FK`, `face_index INTEGER`, `name TEXT`, `mana_cost TEXT`, `type_line TEXT`, `oracle_text TEXT`, `colors INTEGER`, `power TEXT`, `toughness TEXT`, `loyalty TEXT`, `defense TEXT`, `flavor_text TEXT`
  - [x] `printings` table: `id UUID PK`, `oracle_id UUID FK`, `set_code TEXT`, `collector_number TEXT`, `rarity TEXT`, `lang TEXT`, `released_at TEXT`, `image_uris TEXT (JSON)`, `promo BOOLEAN`, `digital BOOLEAN`, `full_art BOOLEAN`, `textless BOOLEAN`, `reprint BOOLEAN`, `prices TEXT (JSON)`
  - [x] Indexes: on `oracle_cards.oracle_id`, `printings.oracle_id`, `printings.set_code`, `card_faces.name`
- [x] Write migration `002_fts5.sql` (behind `fts` feature)
  - [x] Create FTS5 virtual table on `card_faces.name` + `card_faces.oracle_text`
  - [x] Create triggers to keep FTS index in sync on INSERT/UPDATE/DELETE
- [x] Implement `SqliteBackend` struct in `sqlite.rs`
  - [x] Wrap `sqlx::SqlitePool`
  - [x] Implement `StorageBackend` trait (execute, query, transaction)
  - [x] Add `SqliteBackend::open(path)` and `SqliteBackend::open_in_memory()` constructors
  - [x] Run migrations on open: `sqlx::migrate!("./migrations").run(&pool).await?`
- [x] Implement `CardRepository` for `SqliteBackend` in `repo/card.rs`
  - [x] `upsert_oracle()` — INSERT OR REPLACE oracle card + faces
  - [x] `upsert_printing()` — INSERT OR REPLACE printing
  - [x] `get_by_id()` — SELECT printing by Scryfall UUID
  - [x] `get_oracle()` — SELECT oracle card + join faces by oracle_id
  - [x] `search_name()` — LIKE query on card_faces.name
  - [x] `search_fts()` — FTS5 MATCH query (feature-gated)
  - [x] `cards_in_set()` — SELECT printings WHERE set_code = ?
  - [x] `legal_in_format()` — JSON query on legalities column
- [x] Implement `repo/price.rs` — price snapshot queries
- [x] Implement `repo/collection.rs` — collection CRUD (basic structure for Phase 2)
- [x] Add integration tests: open in-memory db → ingest test data → query → verify

## 1B. `elbrus-scryfall` — Bulk Ingest Pipeline

> **Stub conversions:** `ManaCost`, `TypeLine`, and `OracleText` conversions use fallback paths
> here. Unknown/unparsed values wrap raw strings rather than calling `todo!()`, so ingest works
> end-to-end before parsers are implemented. Replace stub paths in 1C.

- [x] Define `ScryfallCard` struct in `models.rs` — mirror Scryfall JSON 1:1
  - [x] All card-level fields: `id`, `oracle_id`, `name`, `layout`, `mana_cost`, `type_line`, `oracle_text`, `colors`, `color_identity`, `keywords`, `legalities`, `set`, `collector_number`, `rarity`, `released_at`, `image_uris`, `prices`, etc.
  - [x] `card_faces: Option<Vec<ScryfallCardFace>>` for multi-face cards
  - [x] `#[serde(default)]` on optional fields for robustness
- [x] Implement `convert.rs` — `ScryfallCard` → `(OracleCard, Printing)` with stub converters
  - [x] Map `ScryfallCard.layout` string → `CardLayout` enum (unknown layouts → `CardLayout::Unknown`)
  - [x] Stub: `mana_cost` string → `ManaCost` via `ManaCost::parse` with `unwrap_or_else` fallback to `ManaSymbol::Unknown(raw)`
  - [x] Stub: `type_line` string → `TypeLine` via `TypeLine::parse` with `unwrap_or_else` fallback to whole string as one `Subtype`
  - [x] Stub: oracle text string → `OracleText` wrapping entire text as single `OracleTextSegment::Text`
  - [x] Map `card_faces` → `SmallVec<[CardFace; 2]>`
  - [x] Map `legalities` HashMap → `Legalities`
  - [x] Map `prices` → `PriceSnapshot` with `rust_decimal` parsing
  - [x] Map `image_uris` → `ImageUris`
  - [x] Handle all card layouts: normal, split, flip, transform, modal_dfc, meld, adventure, etc.
- [x] Implement `BulkIngestor::ingest_file()` — streaming JSON parse
  - [x] Use `serde_json::Deserializer::from_reader(file).into_iter::<ScryfallCard>()` — streams without loading full file into RAM
  - [x] Yield batches of `chunk_size` (default 500) `(OracleCard, Printing)` pairs
  - [x] Track and emit `IngestStats`
- [x] Implement `BulkIngestor::ingest_into_db()` — stream + batch-insert via `CardRepository`
- [x] Add integration test: ingest a small sample JSON file, verify card count and field values

## 1C. `elbrus-cli` — Ingest Smoke Test

> Wire up just enough CLI to validate the full ingest pipeline end-to-end. This is your
> integration test harness before implementing real parsers.

- [x] Add `clap` dependency to `elbrus-cli`
- [x] Implement `elbrus ingest <path>` subcommand — runs `BulkIngestor::ingest_into_db`, prints `IngestStats`
- [x] Download `oracle-cards` bulk export from Scryfall (or use `scripts/download_bulk.py`)
- [x] Ingest full bulk file; verify it completes in < 60s and card counts match Scryfall totals

## 1D. `elbrus-core` — Implement Parser Method Bodies

> Now that real bulk data is flowing, pull edge-case examples from the DB and write failing
> tests _before_ implementing each parser (test-first).

- [ ] `ManaCost::parse(s: &str)` — parse `{W}{U}{2}` notation into `SmallVec<ManaSymbol>`
  - [ ] Handle all `ManaSymbol` variants: colored, generic, variable (X), colorless (C), snow, hybrid, mono-hybrid, phyrexian, hybrid-phyrexian, tap
  - [ ] Handle edge cases: `{X}{X}`, `{CHAOS}`, `{½}`, `{100}`
- [ ] `ManaCost::cmc()` — sum converted mana values of all symbols
  - [ ] Colored/colorless/phyrexian/snow = 1, generic = N, variable = 0, hybrid = max(a,b), mono-hybrid = 2
- [ ] `ManaCost::color_identity()` — union of all colored pips into `ColorSet`
- [ ] `ColorSet::devotion(color)` — count pips of given color in a cost
- [ ] `TypeLine::parse(s: &str)` — split on `—` (em-dash), parse supertypes/types/subtypes
  - [ ] Handle cards with no subtypes, multiple supertypes, unknown types → `Unknown(Arc<str>)`
- [ ] `OracleText::to_display_string()` — concatenate segments back to readable text
- [ ] `Keyword::takes_cost()` — return `true` for Cycling, Kicker, Equip, Morph, Ninjutsu, etc.
- [ ] `Keyword::is_evasion()` — return `true` for Flying, Menace, Intimidate, Fear, Shadow, etc.
- [ ] Add unit tests for all parsers seeded with real edge cases from ingested data
- [ ] Add serde round-trip tests: `serde_json::from_str(serde_json::to_string(&v)) == Ok(v)` for all core types
- [ ] Replace stub converter paths in `convert.rs` with real parser calls (fallbacks remain for true unknowns)

## 1E. End-to-End Validation

- [ ] Download `oracle-cards` bulk export from Scryfall
- [ ] Re-ingest full bulk file with real parsers active; verify counts still match
- [ ] Benchmark: ingest must complete in < 60s
- [ ] Run `cargo fmt --all -- --check` (clean)
- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings` (clean)
- [ ] Run `cargo test --all-features` (all pass)
