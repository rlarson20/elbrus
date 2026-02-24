# Elbrus — Roadmap Continuation Checklist

> Living checklist for tracking implementation progress.
> Mark items: `[ ]` not started · `[/]` in progress · `[x]` done
> Last updated: 2026-02-24

---

## Phase 0 — Repository Hygiene

- [x] Add `LICENSE` file (Apache-2.0 full text) to repo root
- [/] Add `.github/workflows/ci.yml`
  - [x] `cargo fmt --all -- --check`
  - [x] `cargo clippy --all-targets --all-features -- -D warnings`
  - [x] `cargo test --all-features` (with `SQLX_OFFLINE=true`)
  - [x] `cargo build -p elbrus-wasm --target wasm32-unknown-unknown --no-default-features`
  - [x] `maturin build -m crates/elbrus-py/Cargo.toml --no-sdist`
  - [ ] revise and ensure that everything you want to handle is handled
- [ ] Add `sqlx-data.json` offline cache workflow (`cargo sqlx prepare`)
- [ ] Verify entire workspace compiles: `cargo check --workspace --all-features`
- [x] Add `ObjectId(u32)` and `ZoneId(u8)` newtypes to `elbrus-core` (forward-compat for Phase 5)

---

## Phase 1 — Foundation (`elbrus-db` → `elbrus-scryfall` → `elbrus-core`)

> **Build order rationale:** get data moving before implementing parsers. `ManaCost::parse`,
> `TypeLine::parse`, etc. are implemented _after_ ingest works — at that point you have a real
> corpus to write tests against. Use fallback/stub conversions in `convert.rs` until then (see 1B).

### 1A. `elbrus-db` — SQLite Schema & Implementation

- [x] Create `crates/elbrus-db/migrations/` directory
- [x] Write migration `001_initial_schema.sql`
  - [x] `oracle_cards` table: `oracle_id UUID PK`, `layout TEXT`, `color_identity INTEGER`, `keywords TEXT (JSON)`, `legalities TEXT (JSON)`, `edh_rank INTEGER`, `reserved BOOLEAN`
  - [x] `card_faces` table: `oracle_id UUID FK`, `face_index INTEGER`, `name TEXT`, `mana_cost TEXT`, `type_line TEXT`, `oracle_text TEXT`, `colors INTEGER`, `power TEXT`, `toughness TEXT`, `loyalty TEXT`, `defense TEXT`, `flavor_text TEXT`
  - [x] `printings` table: `id UUID PK`, `oracle_id UUID FK`, `set_code TEXT`, `collector_number TEXT`, `rarity TEXT`, `lang TEXT`, `released_at TEXT`, `image_uris TEXT (JSON)`, `promo BOOLEAN`, `digital BOOLEAN`, `full_art BOOLEAN`, `textless BOOLEAN`, `reprint BOOLEAN`, `prices TEXT (JSON)`
  - [x] Indexes: on `oracle_cards.oracle_id`, `printings.oracle_id`, `printings.set_code`, `card_faces.name`
- [x] Write migration `002_fts5.sql` (behind `fts` feature)
  - [x] Create FTS5 virtual table on `card_faces.name` + `card_faces.oracle_text`
  - [x] Create triggers to keep FTS index in sync on INSERT/UPDATE/DELETE
- [ ] Implement `SqliteBackend` struct in `sqlite.rs`
  - [ ] Wrap `sqlx::SqlitePool`
  - [ ] Implement `StorageBackend` trait (execute, query, transaction)
  - [ ] Add `SqliteBackend::open(path)` and `SqliteBackend::open_in_memory()` constructors
  - [ ] Run migrations on open: `sqlx::migrate!("./migrations").run(&pool).await?`
- [ ] Implement `CardRepository` for `SqliteBackend` in `repo/card.rs`
  - [ ] `upsert_oracle()` — INSERT OR REPLACE oracle card + faces
  - [ ] `upsert_printing()` — INSERT OR REPLACE printing
  - [ ] `get_by_id()` — SELECT printing by Scryfall UUID
  - [ ] `get_oracle()` — SELECT oracle card + join faces by oracle_id
  - [ ] `search_name()` — LIKE query on card_faces.name
  - [ ] `search_fts()` — FTS5 MATCH query (feature-gated)
  - [ ] `cards_in_set()` — SELECT printings WHERE set_code = ?
  - [ ] `legal_in_format()` — JSON query on legalities column
- [ ] Implement `repo/price.rs` — price snapshot queries
- [ ] Implement `repo/collection.rs` — collection CRUD (basic structure for Phase 2)
- [ ] Add integration tests: open in-memory db → ingest test data → query → verify

### 1B. `elbrus-scryfall` — Bulk Ingest Pipeline

> **Stub conversions:** `ManaCost`, `TypeLine`, and `OracleText` conversions use fallback paths
> here. Unknown/unparsed values wrap raw strings rather than calling `todo!()`, so ingest works
> end-to-end before parsers are implemented. Replace stub paths in 1C.

- [ ] Define `ScryfallCard` struct in `models.rs` — mirror Scryfall JSON 1:1
  - [ ] All card-level fields: `id`, `oracle_id`, `name`, `layout`, `mana_cost`, `type_line`, `oracle_text`, `colors`, `color_identity`, `keywords`, `legalities`, `set`, `collector_number`, `rarity`, `released_at`, `image_uris`, `prices`, etc.
  - [ ] `card_faces: Option<Vec<ScryfallCardFace>>` for multi-face cards
  - [ ] `#[serde(default)]` on optional fields for robustness
- [ ] Implement `convert.rs` — `ScryfallCard` → `(OracleCard, Printing)` with stub converters
  - [ ] Map `ScryfallCard.layout` string → `CardLayout` enum (unknown layouts → `CardLayout::Unknown`)
  - [ ] Stub: `mana_cost` string → `ManaCost` via `ManaCost::parse` with `unwrap_or_else` fallback to `ManaSymbol::Unknown(raw)`
  - [ ] Stub: `type_line` string → `TypeLine` via `TypeLine::parse` with `unwrap_or_else` fallback to whole string as one `Subtype`
  - [ ] Stub: oracle text string → `OracleText` wrapping entire text as single `OracleTextSegment::Text`
  - [ ] Map `card_faces` → `SmallVec<[CardFace; 2]>`
  - [ ] Map `legalities` HashMap → `Legalities`
  - [ ] Map `prices` → `PriceSnapshot` with `rust_decimal` parsing
  - [ ] Map `image_uris` → `ImageUris`
  - [ ] Handle all card layouts: normal, split, flip, transform, modal_dfc, meld, adventure, etc.
- [ ] Implement `BulkIngestor::ingest_file()` — streaming JSON parse
  - [ ] Use `serde_json::Deserializer::from_reader(file).into_iter::<ScryfallCard>()` — streams without loading full file into RAM
  - [ ] Yield batches of `chunk_size` (default 500) `(OracleCard, Printing)` pairs
  - [ ] Track and emit `IngestStats`
- [ ] Implement `BulkIngestor::ingest_into_db()` — stream + batch-insert via `CardRepository`
- [ ] Add integration test: ingest a small sample JSON file, verify card count and field values

### 1C. `elbrus-cli` — Ingest Smoke Test

> Wire up just enough CLI to validate the full ingest pipeline end-to-end. This is your
> integration test harness before implementing real parsers.

- [ ] Add `clap` dependency to `elbrus-cli`
- [ ] Implement `elbrus ingest <path>` subcommand — runs `BulkIngestor::ingest_into_db`, prints `IngestStats`
- [ ] Download `oracle-cards` bulk export from Scryfall (or use `scripts/download_bulk.py`)
- [ ] Ingest full bulk file; verify it completes in < 60s and card counts match Scryfall totals

### 1D. `elbrus-core` — Implement Parser Method Bodies

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

### 1E. End-to-End Validation

- [ ] Download `oracle-cards` bulk export from Scryfall
- [ ] Re-ingest full bulk file with real parsers active; verify counts still match
- [ ] Benchmark: ingest must complete in < 60s
- [ ] Run `cargo fmt --all -- --check` (clean)
- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings` (clean)
- [ ] Run `cargo test --all-features` (all pass)

---

## Phase 2 — User-Facing (`elbrus-deck` → `elbrus-collection` → `elbrus-rules` P1)

### 2A. `elbrus-deck` — Deck Parsing

- [ ] Implement `ArenaParser`
  - [ ] `can_parse()` — detect Arena export format (e.g., `Deck\n` header or `N CardName` lines)
  - [ ] `parse()` — parse `N CardName (SET) CN` lines into `Deck`
  - [ ] `serialize()` — emit Arena-format text
  - [ ] Handle Commander/Companion sections
- [ ] Implement `MtgoParser`
  - [ ] Parse MTGO `.dek` XML format or text export
  - [ ] Handle sideboard markers
- [ ] Implement `MoxfieldParser`
  - [ ] Parse Moxfield CSV/text export format
  - [ ] Handle Moxfield-specific sections (considering, maybeboard)
- [ ] Add `DeckResolver` — resolve `card_name` → `Uuid` via `CardRepository` lookup
  - [ ] Fuzzy matching for minor name variations
  - [ ] Set hint resolution (prefer matching set code)
- [ ] Deck validation
  - [ ] Format-specific validation: minimum deck size, max copies, banned cards
  - [ ] Commander-specific: color identity check, singleton rule
- [ ] Unit tests for each parser format
- [ ] Round-trip tests: parse → serialize → parse = same `Deck`

### 2B. `elbrus-collection` — Inventory Management

- [ ] Define collection types
  - [ ] `CollectionEntry { printing_id: Uuid, quantity: u32, condition: Condition, foil: bool, notes: Option<Arc<str>> }`
  - [ ] `enum Condition { NearMint, LightlyPlayed, ModeratelyPlayed, HeavyPlayed, Damaged }`
  - [ ] `Collection { entries: Vec<CollectionEntry> }`
- [ ] Implement `CollectionRepository` trait in `elbrus-db`
  - [ ] CRUD: add, update quantity, remove, list
  - [ ] Bulk import from CSV
- [ ] Price snapshot ingest from Scryfall bulk data
  - [ ] Store historical snapshots with `fetched_at` timestamp
  - [ ] Query: current value, price history, total collection value
- [ ] Want-list diffing
  - [ ] `WantList { entries: Vec<WantEntry> }` — cards you want to acquire
  - [ ] `diff(collection, want_list) → Vec<MissingCard>` — what you still need
- [ ] Unit tests + integration tests with in-memory db

### 2C. `elbrus-rules` Phase 1 — Data Only

- [ ] Define types: `ComprehensiveRules`, `RuleNumber`, `Rule`, `KeywordRegistry`, `KeywordDefinition`
- [ ] Implement CR text parser
  - [ ] Parse the Comprehensive Rules plaintext file (from WotC)
  - [ ] Build `BTreeMap<RuleNumber, Rule>` from numbered sections
  - [ ] Extract glossary from the glossary section
  - [ ] Cross-reference `see_also` links between rules
- [ ] Build `KeywordRegistry` from CR section 702 (abilities) and 701 (actions)
  - [ ] Populate `takes_cost`, `is_evasion`, summary for each keyword
- [ ] Format legality + B&R timeline
  - [ ] Define `BanList`, `BanEntry` types
  - [ ] Data source: scrape or manually maintain B&R data
  - [ ] Query: "was card X legal in format Y on date Z?"
- [ ] Unit tests for CR parsing, keyword registry population

---

## Phase 3 — Analysis (`elbrus-analysis` → `elbrus-combos` → `elbrus-draft`)

### 3A. `elbrus-analysis` — Statistical Tools

- [ ] `hypergeometric_pmf(population, successes, draws, exactly) → f64`
- [ ] `hypergeometric_cdf(population, successes, draws, wanted) → f64` — P(draw ≥ wanted)
- [ ] `recommended_land_count()` — Frank Karsten model
  - [ ] Parameters: deck size, avg CMC, cantrip density, target turn/lands
  - [ ] Return recommended land count
- [ ] `ManaCurve` — CMC → count distribution
- [ ] `ManaBaseAnalysis` — full deck mana analysis
  - [ ] Compute curve, avg CMC, recommended lands, color requirements (pip-weighted)
  - [ ] `on_curve_probability: Vec<f64>` — P(N lands by turn N) for turns 1–7
- [ ] `MulliganSimulator` — London mulligan Monte Carlo
  - [ ] `KeepStrategy` trait: `should_keep(&self, hand, on_play, mulligan_count) → bool`
  - [ ] Default strategies: "keep N+ lands", "keep curve playable"
  - [ ] Run N iterations, report keep rate per mulligan depth
- [ ] Use `f64` for all probability calculations (precision requirement)
- [ ] Unit tests with known combinatorial results

### 3B. `elbrus-combos` — Commander Spellbook Integration

- [ ] Define `ComboDatabase`, `Combo` types
- [ ] Ingest Commander Spellbook data (JSON API or bulk export)
  - [ ] Map card names → oracle_ids via db lookup
  - [ ] Store combos with pieces, results, steps, color identity
- [ ] `find_enabled_combos(pool) → Vec<&Combo>` — all combos possible with given cards
- [ ] `find_near_combos(pool, pieces_missing) → Vec<(&Combo, Vec<Uuid>)>` — combos needing ≤ N more pieces
- [ ] Integration tests with sample combo data

### 3C. `elbrus-draft` — Draft Simulation

- [ ] Define draft types
  - [ ] `BoosterPack { cards: Vec<Uuid> }` — generated from set data
  - [ ] `DraftState { players: Vec<DraftPlayer>, round: u32, direction: Direction }`
  - [ ] `DraftPlayer { pool: Vec<Uuid>, current_pack: Option<BoosterPack> }`
- [ ] `PickStrategy` trait: `pick(&self, pack: &[Uuid], pool: &[Uuid]) → usize`
  - [ ] Default strategies: random, rarity-based, color-signal
- [ ] Booster generation from set card pool (respecting rarity slots)
- [ ] Cube draft support (custom card pool, no rarity slots)
- [ ] Draft state machine: open pack → pick → pass → repeat
- [ ] Unit tests for pack generation, pick strategies, state transitions

---

## Phase 4 — Interfaces (`elbrus-cli` → `elbrus-py` → `elbrus-wasm`)

### 4A. `elbrus-cli` — Command-Line Interface

- [ ] Add `clap` dependency for argument parsing
- [ ] Subcommands:
  - [ ] `elbrus ingest <path>` — run bulk Scryfall ingest
  - [ ] `elbrus search <query>` — search cards by name/text
  - [ ] `elbrus card <name-or-uuid>` — show card details
  - [ ] `elbrus deck analyze <file>` — parse deck, show mana analysis
  - [ ] `elbrus deck validate <file> --format <fmt>` — check legality
  - [ ] `elbrus collection diff <collection> <wantlist>` — show missing cards
  - [ ] `elbrus combo check <deck-or-pool>` — find combos in card pool
- [ ] Pretty output: colored terminal output, tables for data
- [ ] Error handling: `anyhow` at the binary boundary
- [ ] Integration tests for each subcommand

### 4B. `elbrus-py` — Python Bindings

- [ ] Embed `tokio::Runtime` in a `#[pyclass] ElbrusDb` struct
- [ ] Core methods (all `block_on` at boundary, release GIL during async):
  - [ ] `ElbrusDb.open(path: str) → ElbrusDb`
  - [ ] `ElbrusDb.search_name(query: str, limit: int) → list[dict]`
  - [ ] `ElbrusDb.search_text(query: str, limit: int) → list[dict]`
  - [ ] `ElbrusDb.get_card(uuid: str) → dict`
  - [ ] `ElbrusDb.ingest(path: str) → dict` (IngestStats)
- [ ] Bridge types via `serde_json → json.loads()` (avoid per-type `ToPyObject`)
- [ ] `__repr__` / `__eq__` via `Debug` / `PartialEq` derives
- [ ] Pickle support via `__getstate__`/`__setstate__` with `serde_json`
- [ ] Type stubs (`.pyi` file) for IDE autocomplete
- [ ] `maturin` build config: `pyproject.toml` or `Cargo.toml` metadata
- [ ] Test: `pip install -e .` then `import elbrus` smoke test

### 4C. `elbrus-wasm` — Browser Bindings

- [ ] Expose search/analysis functions via `#[wasm_bindgen]`
  - [ ] `search_name(query: &str) → JsValue` (serialized results)
  - [ ] `analyze_deck(deck_text: &str) → JsValue` (mana analysis)
  - [ ] `hypergeometric(pop, succ, draws, wanted) → f64`
- [ ] Use `serde-wasm-bindgen` for type marshalling
- [ ] WASM `StorageBackend` design
  - [ ] JS shim for `wa-sqlite` + OPFS
  - [ ] Or: accept pre-built SQLite as `Uint8Array`, load in-memory
- [ ] Build with `wasm-pack`
  - [ ] `wasm-pack build crates/elbrus-wasm --target web`
  - [ ] Verify bundle size is acceptable (< 1MB gzipped target)
- [ ] Publish as npm package on tag
- [ ] Smoke test in a minimal HTML page

---

## Phase 5 — Rules Simulation (`elbrus-rules` Phase 2)

### 5A. Game Object Model

- [ ] Define zone model
  - [ ] `ZoneId(u8)` enum: Library, Hand, Battlefield, Graveyard, Stack, Exile, Command
  - [ ] `GameState { objects: SlotMap<ObjectKey, GameObj>, zones: HashMap<ZoneId, Vec<ObjectKey>>, stack: Vec<StackEntry> }`
- [ ] Define `GameObj` — unified object for cards/tokens/copies on stack
  - [ ] Current characteristics (name, types, P/T, abilities — may differ from printed)
  - [ ] Controller, owner, timestamps, counters
- [ ] `StackEntry` — object + snapshot of state at time of casting
- [ ] Zone transfer API: `move_object(obj, from_zone, to_zone) → Result`

### 5B. Core Game Actions

- [ ] Turn structure state machine (untap → upkeep → draw → main1 → combat → main2 → end)
- [ ] Priority system (APNAP order)
- [ ] Casting spells: move to stack, pay costs, resolve
- [ ] Activated abilities: pay cost, put on stack, resolve
- [ ] Triggered abilities: trigger → go on stack in APNAP order

### 5C. State-Based Actions

- [ ] Creature dies (toughness ≤ 0 or lethal damage)
- [ ] Player loses (life ≤ 0, can't draw, poison ≥ 10)
- [ ] Legend rule, planeswalker uniqueness
- [ ] Token removal from non-battlefield zones
- [ ] +1/+1 and -1/-1 counter annihilation

### 5D. Testing & Determinism

- [ ] Deterministic game replay: seed RNG, record all choices → replay produces identical state
- [ ] Golden-file tests: known game sequences produce known final states
- [ ] Fuzz testing: random valid action sequences don't panic

---

## Stretch Goals (No Phase Assigned)

- [ ] Legality timeline viewer
- [ ] B&R change tracking with announcement URLs
- [ ] Sealed pool evaluator (open 6 packs → recommend 40-card deck)
- [ ] Format staples index (most-played cards per format)
- [ ] EDH bracket estimator (power level classification)
- [ ] Price spike alerts (compare snapshots, flag large % changes)
- [ ] Proxy PDF generator (text-only, no art — legal for playtest)
- [ ] Card aging / reprint tracker (time since last print, reprint probability)
- [ ] "Cards like this" oracle text similarity search (TF-IDF or embedding-based)
