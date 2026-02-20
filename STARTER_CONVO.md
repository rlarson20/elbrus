[full source](https://claude.ai/chat/a03c796b-0123-4619-896c-fedda50ee299)

## TL;DR

- **Monorepo of focused crates** (`mtg-core`, `mtg-scryfall`, `mtg-deck`, `mtg-rules`, `mtg-draft`, `mtg-collection`, `mtg-py`, `mtg-wasm`)
- `mtg-core` owns canonical types; everything else depends on it
- Offline-first via SQLite + Scryfall bulk JSON; network is a sync mechanism, not runtime dependency
- PyO3 and WASM are thin binding layers, not logic holders
- Rules engine starts as oracle text parser + legality validator; full sim is a later milestone

---

## Crate Map

```
mtg/
├── crates/
│   ├── mtg-core/          # Canonical types, traits, errors. Zero network deps.
│   ├── mtg-scryfall/      # Bulk data ingestion + live API client (optional feature)
│   ├── mtg-db/            # SQLite persistence layer (rusqlite / sqlx). Owns migrations.
│   ├── mtg-deck/          # Parsing: Arena, MTGO, Moxfield. Validation against db.
│   ├── mtg-rules/         # CR text, keyword registry, legality checks. Sim later.
│   ├── mtg-draft/         # Cube/booster pack logic, pick history, draft state.
│   ├── mtg-collection/    # Inventory, pricing (TCGPlayer/Scryfall price blobs).
│   ├── mtg-cli/           # Binary. Thin shell over the crates above.
│   ├── mtg-py/            # PyO3 bindings crate. Exposes python wheels.
│   └── mtg-wasm/          # wasm-bindgen bindings. Exposes JS/TS API.
└── Cargo.toml             # workspace
```

---

## Crate Detail

### `mtg-core`

The shared vocabulary. No I/O, no network, no DB.

```
Card, Face, CardLayout, Color, ColorIdentity, ManaCost, ManaSymbol
Rarity, Legality, Format, Supertype, Cardtype, Subtype
OracleText, Keyword (enum, exhaustive)
SetCode, CollectorNumber, ScryfallId (newtype UUIDs)
Ruling, Price, ImageUris
Error (thiserror), Result<T>
```

Traits to define here:

- `Searchable` — filter predicate interface
- `Printable` — canonical display (name + set + cn)
- `Legality` — `is_legal_in(format: Format) -> bool`

### `mtg-scryfall`

- Ingest bulk JSON (`default-cards`, `all-cards`, `rulings`) → deserialize into `mtg-core` types
- Live API client behind `feature = "live"` (reqwest + tokio); off by default
- Bulk download + incremental update (etag/modified-since)
- Maps Scryfall JSON schema → core types; Scryfall-specific fields stay in a `ScryfallMeta` wrapper so core stays clean

### `mtg-db`

- SQLite via `sqlx` (async, compile-time checked queries)
- Schema owns: cards, faces, sets, rulings, prices, collection entries, cached search results
- Migration runner built in (`sqlx::migrate!`)
- Exposes a `MtgDb` handle; other crates take `&MtgDb`, never raw SQL
- FTS5 virtual table for card name / oracle text search

### `mtg-deck`

- Parsers for: Arena export, MTGO `.dek`, Moxfield plaintext, generic "1x Card Name (SET) CN"
- Output: `Decklist { main: Vec<DeckEntry>, side: Vec<DeckEntry>, commander: Option<DeckEntry> }`
- Validation: legal set membership, deck size rules per format, commander identity
- Serialization back to each format

### `mtg-rules`

**Phase 1 (build this first):**

- Comprehensive Rules (CR) text as structured data (parse the TXT WotC publishes)
- Keyword registry: which keywords exist, reminder text, whether they're evergreen
- Format legality: given a decklist + `MtgDb`, return legality per format + violations

**Phase 2 (later):**

- Zone model, object model (permanent, spell, ability)
- Stack + priority
- Full APNAP resolution

Phase 2 is a multi-year effort. Design Phase 1 types so Phase 2 can grow into them without breaking the API. The key insight: **don't couple the sim to Scryfall data** — it needs to work with abstract game objects.

### `mtg-draft`

- `BoosterPack` generation from set data (respects slot rules, foil rates)
- `DraftState`: seat → picks, pack queue, pick history
- Cube list import/export, randomized pack generation from cube
- Hook interface for draft bots / pick evaluators (trait `PickStrategy`)

### `mtg-collection`

- Inventory: `(card_id, finish, condition, quantity, purchase_price)`
- Price sync from Scryfall price blobs (already in bulk data)
- Collection value over time (append-only price snapshots)
- Want list + "do I own this decklist?" diff

### `mtg-py` / `mtg-wasm`

- Thin. All logic lives in the crates above.
- PyO3: expose `MtgDb`, `Decklist`, search/filter, collection API. Generate stubs.
- WASM: target the subset that's useful in browser (no SQLite unless OPFS) — deck parsing, card lookup via HTTP, draft state

---

## Data Flow

```
Scryfall bulk JSON
       │
  mtg-scryfall (deserialize + normalize)
       │
  mtg-db (SQLite, FTS5)
       │
  ┌────┴─────────────────────────┐
  │              │               │
mtg-deck    mtg-draft    mtg-collection
  │
mtg-rules (validation)
  │
mtg-cli / mtg-py / mtg-wasm
```

---

## Build Order

| Phase | Crates                   | Milestone                                    |
| ----- | ------------------------ | -------------------------------------------- |
| 0     | `mtg-core`               | Stable types; semver locked early            |
| 1     | `mtg-scryfall`, `mtg-db` | Bulk ingest works, card search works         |
| 2     | `mtg-deck`               | Parse all 3 formats, validate legality       |
| 3     | `mtg-collection`         | Inventory + pricing                          |
| 4     | `mtg-rules` Phase 1      | CR parser, keyword registry, format legality |
| 5     | `mtg-draft`              | Cube + booster draft state                   |
| 6     | `mtg-py`, `mtg-wasm`     | Bindings after API is stable                 |
| 7     | `mtg-rules` Phase 2      | Sim, zone model, stack                       |

---

## Key Decisions to Lock Early

**`mtg-core` types must be stable before Phase 3.** Changing `Card` after PyO3 bindings exist is painful. Consider `#[non_exhaustive]` on enums you know will grow (like `Keyword`).

**SQLx vs rusqlite:** SQLx gives async + compile-time query checks; rusqlite is simpler and sync. Given offline-first + CLI use, rusqlite is fine for v0. SQLx if you expect concurrent async access.

**Scryfall UUID as primary key everywhere.** Don't invent your own. Scryfall's `id` is stable per printing.

**Don't model Oracle text as a string.** Even in Phase 1, start a `ParsedOracleText` type that's a `Vec<OracleTextSegion>` (keyword, cost, reminder, flavor, etc.). Retrofitting this later is brutal.

---

## Things You're Probably Missing

- **Mana evaluation / CMC utilities** — goldfish damage, curve analysis, mana base calculators. Fits in `mtg-deck` or a `mtg-analysis` crate.
- **Set/block/release calendar** — useful for "what was in standard on date X" queries. Scryfall has this data.
- **Token/emblem/counter data** — Scryfall bulk includes tokens; worth modeling.
- **Art/image pipeline** — download + cache card art; separate crate, but people will want it.
- **Event/tournament data** — not in Scryfall; would need MTGTOP8 or MTGGoldfish scraping. Risky/fragile but high value.

Want me to flesh out any specific crate's API surface or start with the `Cargo.toml` workspace skeleton + `mtg-core` type definitions?
