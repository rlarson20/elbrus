# Convo

## Prompt

I'm building elbrus, a Rust workspace of MtG developer libraries. It should be fast, correct, and accessible to Rust, Python (PyO3), and WASM consumers. Everything is offline-first: Scryfall bulk JSON ingested into a local SQLite database; network is a sync mechanism only.
Workspace crates:

- `elbrus-core` — canonical types/traits, zero I/O deps
- `elbrus-scryfall` — bulk ingest + optional live API (`feature = "live"`)
- `elbrus-db` — SQLite via sqlx, FTS5, owns all migrations
- `elbrus-deck` — parse/validate Arena, MTGO, Moxfield formats
- `elbrus-rules` — CR text parser, keyword registry, format legality (sim later)
- `elbrus-draft` — booster/cube draft state, `PickStrategy` trait
- `elbrus-collection` — inventory, price snapshots, want-list diffing
- `elbrus-analysis` — hypergeometric calc, mana curve/base analysis, mulligan sim, Frank Karsten math
- `elbrus-combos` — CommanderSpellbook integration, combo piece detection
- `elbrus-cli` — thin binary shell over all crates
- `elbrus-py` — PyO3 bindings, wheel builds via maturin
- `elbrus-wasm` — wasm-bindgen bindings, browser-safe subset
  Additional planned tools: legality timeline, B&R change tracking, sealed pool evaluator, format staples index, EDH bracket estimator, price spike alerts, proxy PDF generator (text-only, no art), card aging/reprint tracker, "cards like this" oracle text similarity search.
  Key design constraints:
- `elbrus-core` types must stabilize before binding layers. Use `#[non_exhaustive]` on open enums (Keyword, Subtype, etc.)
- `OracleText` is `Vec<OracleTextSegment>` from day one, never a raw string
- Scryfall UUID is the canonical primary key everywhere
- `elbrus-py` and `elbrus-wasm` are thin — zero logic lives there
- Rules sim (zone model, stack, APNAP) is Phase 2+; Phase 1 types must accommodate it without breaking changes
- Prefer `sqlx` over `rusqlite` for async + compile-time query checks
- All public types derive `Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize`; in general, follow [this checklist](https://rust-lang.github.io/api-guidelines/checklist.html), idiomatic Rust guidelines, and general Rust best practices.
- Workspace-level dependency pinning in `[workspace.dependencies]`
- CI: fmt, clippy -D warnings, test, wasm-build, maturin dry-run; publish on tag
- License: Apache-2.0
  Build order: core → scryfall + db → deck → collection → rules Phase 1 → analysis + combos → draft → cli → py + wasm → rules Phase 2
  Give me a complete A-Z implementation roadmap: phases, milestones, crate-by-crate API sketches, key type definitions for `elbrus-core`, critical decisions to lock early, and known hard problems to design around (rules engine object model, WASM SQLite constraints, PyO3 GIL considerations, etc.).
  Include concrete Rust type stubs and a working Cargo workspace skeleton.

## Assistant Response

### Elbrus: A-Z Implementation Roadmap

#### TL;DR

- **5 phases**, gated on type stability; Phase 1 is the investment that makes everything else cheap
- Lock `elbrus-core` types _before_ writing a single `sqlx::query!` macro — compile-time query checks bite you if the schema drifts
- WASM SQLite is the gnarliest constraint; design `elbrus-db` with a pluggable storage trait so WASM can swap to `absurd-sql`/OPFS without touching business logic
- PyO3 GIL: keep bindings `Send + Sync` by making all core types `Clone`-cheap; never hold Python objects in Rust structs
- `OracleText` as `Vec<OracleTextSegment>` from day one is the right call — retrofitting this is painful

---

#### Phase Map

```
Phase 1 (Foundation)     core → scryfall → db
Phase 2 (User-Facing)    deck → collection → rules-P1
Phase 3 (Analysis)       analysis → combos → draft
Phase 4 (Interface)      cli → py → wasm
Phase 5 (Rules Sim)      rules-P2 (zone/stack model)
```

---

#### Critical Early Decisions (Lock These First)

| Decision         | Choice                                                                         | Rationale                                       |
| ---------------- | ------------------------------------------------------------------------------ | ----------------------------------------------- |
| Primary key      | Scryfall UUID (`uuid::Uuid`)                                                   | Stable, content-addressed                       |
| Oracle text repr | `Vec<OracleTextSegment>`                                                       | Enables rules parsing, FTS, diff                |
| Async runtime    | `tokio` workspace-wide                                                         | sqlx async, consistent executor                 |
| Error strategy   | `thiserror` per-crate, `anyhow` at binary/binding layer                        | Structured errors propagate cleanly             |
| Serde roundtrip  | All core types must `serde_json::from_str(serde_json::to_string(&v)) == Ok(v)` | Required for WASM postMessage and Python pickle |
| WASM storage     | Abstract `StorageBackend` trait in `elbrus-db`                                 | OPFS/absurd-sql swap without API break          |
| Keyword registry | `#[non_exhaustive] enum Keyword` + `UnknownKeyword(Arc<str>)` variant          | CR adds keywords; don't break consumers         |
| Feature flags    | `live` on scryfall, `fts` on db, `sim` on rules                                | Keep WASM bundle minimal                        |

---

#### Cargo Workspace Skeleton

```toml
### Cargo.toml (root)
[workspace]
resolver = "2"
members = [
    "crates/elbrus-core",
    "crates/elbrus-scryfall",
    "crates/elbrus-db",
    "crates/elbrus-deck",
    "crates/elbrus-rules",
    "crates/elbrus-draft",
    "crates/elbrus-collection",
    "crates/elbrus-analysis",
    "crates/elbrus-combos",
    "crates/elbrus-cli",
    "crates/elbrus-py",
    "crates/elbrus-wasm",
]

[workspace.dependencies]
### Core
uuid       = { version = "1", features = ["v4", "serde"] }
serde      = { version = "1", features = ["derive", "rc"] }
serde_json = "1"
thiserror  = "2"
anyhow     = "1"
chrono     = { version = "0.4", features = ["serde"] }
rust_decimal = { version = "1", features = ["serde-with-str"] }
arc-swap   = "1"
smallvec   = { version = "1", features = ["serde"] }
indexmap   = { version = "2", features = ["serde"] }

### Async
tokio      = { version = "1", features = ["full"] }
futures    = "0.3"

### DB
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "uuid", "chrono", "macros"] }

### HTTP (optional/live)
reqwest = { version = "0.12", features = ["json", "stream"], default-features = false }

### Bindings
pyo3       = { version = "0.23", features = ["extension-module"] }
wasm-bindgen = "0.2"
js-sys     = "0.3"
serde-wasm-bindgen = "0.6"

[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
all      = { level = "deny", priority = -1 }
pedantic = { level = "warn", priority = -1 }
```

```
crates/
├── elbrus-core/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── card.rs
│       ├── oracle.rs
│       ├── color.rs
│       ├── mana.rs
│       ├── keyword.rs
│       ├── types.rs        ### CardType, Supertype, Subtype
│       ├── legality.rs
│       ├── ruling.rs
│       └── error.rs
├── elbrus-scryfall/
│   ├── Cargo.toml          ### features = ["live"], depends on elbrus-db
│   └── src/
│       ├── lib.rs
│       ├── bulk.rs         ### streaming JSON ingest
│       ├── models.rs       ### ScryfallCard (raw deserialization target)
│       ├── convert.rs      ### ScryfallCard → core::Card
│       └── api.rs          ### feature = "live"
├── elbrus-db/
│   ├── Cargo.toml          ### features = ["fts"]
│   ├── migrations/
│   └── src/
│       ├── lib.rs
│       ├── backend.rs      ### StorageBackend trait
│       ├── sqlite.rs
│       ├── repo/
│       │   ├── card.rs
│       │   ├── collection.rs
│       │   └── price.rs
│       └── fts.rs          ### feature = "fts"
### ... etc
```

---

#### `elbrus-core` Type Stubs

##### `color.rs`

```rust
use serde::{Deserialize, Serialize};

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct ColorSet: u8 {
        const WHITE = 0b00001;
        const BLUE  = 0b00010;
        const BLACK = 0b00100;
        const RED   = 0b01000;
        const GREEN = 0b10000;
    }
}

impl ColorSet {
    pub fn is_colorless(self) -> bool { self.is_empty() }
    pub fn is_multicolor(self) -> bool { self.bits().count_ones() > 1 }
    pub fn devotion(self, color: ColorSet) -> u8 { todo!() }
}

/// Pips in a mana cost, in WUBRG order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Color { W, U, B, R, G }
```

##### `mana.rs`

```rust
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

/// A single symbol: {W}, {2/U}, {X}, {T}, {C}, {CHAOS}, etc.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ManaSymbol {
    Colored(Color),
    Generic(u32),
    Variable,           // X
    Colorless,          // C
    Snow,               // S
    Hybrid(Color, Color),
    MonoHybrid(Color),  // {2/W}
    Phyrexian(Color),   // {W/P}
    HybridPhyrexian(Color, Color),
    Tap,                // {T} (for reminder text)
    Unknown(Arc<str>),
}

/// Mana cost as ordered sequence of symbols.
/// Use SmallVec<[_; 8]>: typical spell costs fit on stack.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct ManaCost(pub SmallVec<[ManaSymbol; 8]>);

impl ManaCost {
    pub fn cmc(&self) -> f32 { todo!() }
    pub fn color_identity(&self) -> ColorSet { todo!() }
    pub fn parse(s: &str) -> Result<Self, crate::error::CoreError> { todo!() }
}

use std::sync::Arc;
use crate::{color::Color};
```

##### `oracle.rs`

```rust
use serde::{Deserialize, Serialize};
use crate::mana::ManaCost;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum OracleTextSegment {
    /// Plain prose text (trimmed sentence fragments between other segments).
    Text(Arc<str>),
    /// Inline mana symbol: {W}, {2/B}, etc.
    ManaCost(ManaCost),
    /// Keyword ability with optional cost/parameters.
    Keyword {
        keyword: crate::keyword::Keyword,
        parameter: Option<Arc<str>>,
    },
    /// Reminder text in parentheses.
    Reminder(Arc<str>),
    /// Loyalty/energy/counter symbols.
    Symbol(Arc<str>),
    /// Section separator — double newline in Scryfall JSON.
    Paragraph,
}

/// Structured oracle text. Never a raw String in public API.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct OracleText(pub Vec<OracleTextSegment>);

impl OracleText {
    /// Lossy round-trip for display; do NOT use as canonical form.
    pub fn to_display_string(&self) -> String { todo!() }
    pub fn keywords(&self) -> impl Iterator<Item = &crate::keyword::Keyword> {
        self.0.iter().filter_map(|s| {
            if let OracleTextSegment::Keyword { keyword, .. } = s { Some(keyword) } else { None }
        })
    }
}

use std::sync::Arc;
```

##### `keyword.rs`

```rust
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Comprehensive Rules keyword abilities and keyword actions.
/// #[non_exhaustive] because Wizards adds keywords every set.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Keyword {
    // Keyword abilities (CR 702)
    Deathtouch, Defender, DoubleStrike, Enchant, Equip,
    FirstStrike, Flash, Flying, Haste, Hexproof,
    Indestructible, Intimidate, Landwalk, Lifelink, Menace,
    Protection, Reach, Shroud, Trample, Vigilance,
    Infect, Wither, Persist, Undying, Riot,
    Cascade, Convoke, Delve, Emerge, Escape,
    Foretell, Jumpstart, Kicker, Mutate, Overload,
    Replicate, Spectacle, Surge, Transmute, Unearth,
    Improvise, Affinity, Aftermath, Bestow, Cycling,
    Dash, Evoke, Flashback, Madness, Miracle,
    Morph, Ninjutsu, Prowl, Suspend, Transfigure,
    Ward, Ravenous, Squad,
    // Keyword actions (CR 701) — subset needed for segment parsing
    Scry, Surveil, Mill, Investigate, Explore,
    // Fallback for future/unknown keywords
    Unknown(Arc<str>),
}

impl Keyword {
    pub fn takes_cost(&self) -> bool { todo!() }
    pub fn is_evasion(&self) -> bool { todo!() }
}
```

##### `types.rs`

```rust
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use smallvec::SmallVec;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Supertype { Basic, Legendary, Snow, World, Token, Unknown(Arc<str>) }

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CardType {
    Artifact, Battle, Conspiracy, Creature, Dungeon, Enchantment,
    Instant, Land, Phenomenon, Plane, Planeswalker, Scheme,
    Sorcery, Tribal, Vanguard, Unknown(Arc<str>),
}

/// Subtype is an open set — Wizards adds creature types, land types, spell types freely.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Subtype(pub Arc<str>);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct TypeLine {
    pub supertypes: SmallVec<[Supertype; 2]>,
    pub card_types: SmallVec<[CardType; 2]>,
    pub subtypes:   SmallVec<[Subtype; 4]>,
}

impl TypeLine {
    pub fn is_creature(&self) -> bool {
        self.card_types.contains(&CardType::Creature)
    }
    pub fn is_permanent(&self) -> bool {
        self.card_types.iter().any(|t| matches!(
            t, CardType::Artifact | CardType::Battle | CardType::Creature |
               CardType::Enchantment | CardType::Land | CardType::Planeswalker
        ))
    }
    pub fn parse(s: &str) -> Result<Self, crate::error::CoreError> { todo!() }
}
```

##### `legality.rs`

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LegalityStatus { Legal, Restricted, Banned, NotLegal }

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Format(pub std::sync::Arc<str>); // open set

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Legalities(pub HashMap<Format, LegalityStatus>);

impl Legalities {
    pub fn is_legal_in(&self, format: &Format) -> bool {
        self.0.get(format).copied() == Some(LegalityStatus::Legal)
    }
}
```

##### `card.rs`

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;
use crate::{
    color::ColorSet,
    legality::Legalities,
    mana::ManaCost,
    oracle::OracleText,
    types::TypeLine,
};

/// Canonical card face. A split/MDFC card has 2 of these.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CardFace {
    pub name:       Arc<str>,
    pub mana_cost:  Option<ManaCost>,
    pub type_line:  TypeLine,
    pub oracle_text: OracleText,
    pub colors:     ColorSet,
    pub power:      Option<Arc<str>>,   // Keep as str: */2+*, etc.
    pub toughness:  Option<Arc<str>>,
    pub loyalty:    Option<Arc<str>>,
    pub defense:    Option<Arc<str>>,   // Battle defense
    pub flavor_text: Option<Arc<str>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CardLayout {
    Normal, Split, Flip, Transform, ModalDfc, Meld, Leveler,
    Class, Saga, Adventure, Prototype, Battle, Mutate,
    Token, DoubleFacedToken, Emblem, Augment, Host, ArtSeries,
    Unknown(Arc<str>),
}

/// The Oracle card — one record per unique oracle_id.
/// Printing-specific data (set, collector number, art, prices) lives in `Printing`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleCard {
    /// Scryfall oracle_id — stable across printings.
    pub oracle_id:       Uuid,
    pub layout:          CardLayout,
    pub faces:           smallvec::SmallVec<[CardFace; 2]>,
    pub color_identity:  ColorSet,
    pub keywords:        Vec<crate::keyword::Keyword>,
    pub legalities:      Legalities,
    pub edh_rank:        Option<u32>,
    pub reserved:        bool,
}

impl OracleCard {
    pub fn primary_face(&self) -> &CardFace { &self.faces[0] }
    pub fn name(&self) -> &str { &self.primary_face().name }
    pub fn cmc(&self) -> f32 {
        self.primary_face().mana_cost.as_ref().map_or(0.0, |m| m.cmc())
    }
}

/// A specific printing — one record per Scryfall card id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Printing {
    /// Scryfall card id — the canonical PK throughout elbrus.
    pub id:             Uuid,
    pub oracle_id:      Uuid,
    pub set_code:       Arc<str>,
    pub collector_number: Arc<str>,
    pub rarity:         Rarity,
    pub lang:           Arc<str>,
    pub released_at:    chrono::NaiveDate,
    pub image_uris:     Option<ImageUris>,
    pub promo:          bool,
    pub digital:        bool,
    pub full_art:       bool,
    pub textless:       bool,
    pub reprint:        bool,
    pub prices:         Option<PriceSnapshot>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum Rarity { Common, Uncommon, Rare, Mythic, Special, Bonus }

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ImageUris {
    pub small:      Option<Arc<str>>,
    pub normal:     Option<Arc<str>>,
    pub large:      Option<Arc<str>>,
    pub png:        Option<Arc<str>>,
    pub art_crop:   Option<Arc<str>>,
    pub border_crop: Option<Arc<str>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PriceSnapshot {
    pub usd:        Option<rust_decimal::Decimal>,
    pub usd_foil:   Option<rust_decimal::Decimal>,
    pub eur:        Option<rust_decimal::Decimal>,
    pub tix:        Option<rust_decimal::Decimal>,
    pub fetched_at: chrono::DateTime<chrono::Utc>,
}
```

---

#### Crate-by-Crate API Sketches

##### `elbrus-scryfall`

```rust
// bulk.rs — streaming ingest, zero full-parse-into-RAM
pub struct BulkIngestor {
    pub chunk_size: usize,  // default 500, batch insert
}

impl BulkIngestor {
    /// Stream-parse a `bulk-data` JSON file, yield batches.
    pub fn ingest_file(
        &self,
        path: &Path,
    ) -> impl Stream<Item = Result<Vec<(OracleCard, Printing)>, ScryfallError>>;

    /// Convenience: ingest directly into db.
    pub async fn ingest_into_db(
        &self, path: &Path, db: &dyn CardRepository,
    ) -> Result<IngestStats, ScryfallError>;
}

pub struct IngestStats {
    pub cards_processed: u64,
    pub cards_inserted:  u64,
    pub cards_updated:   u64,
    pub duration:        std::time::Duration,
}

// models.rs — raw Scryfall shape, separate from core types
// ScryfallCard mirrors the JSON 1:1; convert.rs maps it to (OracleCard, Printing)
```

Key decision: parse Scryfall JSON into a `ScryfallCard` (mirrors their API faithfully) and then `convert` to core types. Never deserialize directly into `OracleCard`. This isolates you from Scryfall schema changes.

##### `elbrus-db`

```rust
// backend.rs
#[async_trait::async_trait]
pub trait StorageBackend: Send + Sync {
    async fn execute(&self, sql: &str, params: &[Value]) -> Result<u64, DbError>;
    async fn query(&self, sql: &str, params: &[Value]) -> Result<Vec<Row>, DbError>;
    async fn transaction<F, T>(&self, f: F) -> Result<T, DbError>
    where F: FnOnce(&mut dyn Transaction) -> BoxFuture<'_, Result<T, DbError>> + Send;
}

// repo/card.rs
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
```

**WASM note**: `StorageBackend` lets the WASM crate inject an OPFS-backed impl. The `sqlx`-based `SqliteBackend` is the native impl. For WASM, you'll likely want to use [`wa-sqlite`](https://github.com/rhashimoto/wa-sqlite) with a JS-side adapter bridged via `wasm-bindgen`. Design the WASM storage adapter as a JS shim that implements `StorageBackend` via `js_sys::Promise` interop. Accept that WASM storage will be asynchronous and sync-over-async hacks will not work.

##### `elbrus-deck`

```rust
pub struct Deck {
    pub name:     Option<Arc<str>>,
    pub format:   Option<Format>,
    pub mainboard: Vec<DeckEntry>,
    pub sideboard: Vec<DeckEntry>,
    pub commander: Vec<DeckEntry>,  // EDH
    pub companion: Option<DeckEntry>,
}

pub struct DeckEntry {
    pub quantity: u32,
    pub card_name: Arc<str>,
    pub resolved:  Option<Uuid>,    // filled after db lookup
    pub set_hint:  Option<Arc<str>>,
    pub foil:      bool,
}

pub trait DeckParser: Send + Sync {
    fn can_parse(&self, input: &str) -> bool;
    fn parse(&self, input: &str) -> Result<Deck, DeckError>;
    fn serialize(&self, deck: &Deck) -> String;
}

pub struct ArenaParser;
pub struct MtgoParser;
pub struct MoxfieldParser;  // URL-fetched or exported text

impl DeckParser for ArenaParser { ... }
```

##### `elbrus-analysis`

```rust
// Hypergeometric: P(drawing ≥ k copies of N successes in pop M, sample n)
pub fn hypergeometric_cdf(
    population: u32, successes: u32, draws: u32, wanted: u32,
) -> f64;

pub fn hypergeometric_pmf(
    population: u32, successes: u32, draws: u32, exactly: u32,
) -> f64;

// Frank Karsten land count recommendation
pub fn recommended_land_count(
    deck_size: u32,
    avg_cmc: f32,
    cantrip_density: f32,  // % of spells that draw a card
    target_turn: u32,      // turn you want N lands
    target_lands: u32,
) -> u32;

pub struct ManaCurve(pub BTreeMap<u32, u32>); // cmc → count

pub struct ManaBaseAnalysis {
    pub curve:             ManaCurve,
    pub avg_cmc:           f32,
    pub recommended_lands: u32,
    pub color_requirements: HashMap<Color, f32>, // pip-weighted
    pub on_curve_probability: Vec<f64>,          // turn 1..7
}

// Mulligan sim (London)
pub struct MulliganSimulator {
    pub iterations: u32,
    pub deck: Vec<Uuid>,  // db resolved
    pub strategy: Box<dyn KeepStrategy>,
}

pub trait KeepStrategy: Send + Sync {
    fn should_keep(&self, hand: &[Uuid], on_play: bool, mulligan_count: u32) -> bool;
}
```

##### `elbrus-rules`

```rust
// Phase 1: data only, no simulation
pub struct ComprehensiveRules {
    pub rules: BTreeMap<RuleNumber, Rule>,
    pub glossary: HashMap<Arc<str>, Arc<str>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuleNumber(pub Arc<str>); // "702.2a" etc.

pub struct Rule {
    pub number:    RuleNumber,
    pub text:      Arc<str>,
    pub see_also:  Vec<RuleNumber>,
}

pub struct KeywordRegistry {
    pub entries: HashMap<Keyword, KeywordDefinition>,
}

pub struct KeywordDefinition {
    pub keyword:      Keyword,
    pub rule:         RuleNumber,
    pub takes_cost:   bool,
    pub is_evasion:   bool,
    pub summary:      Arc<str>,
}

// Format legality timeline
pub struct BanList {
    pub format:   Format,
    pub entries:  Vec<BanEntry>,
}

pub struct BanEntry {
    pub oracle_id:  Uuid,
    pub status:     LegalityStatus,
    pub effective:  chrono::NaiveDate,
    pub announcement_url: Option<Arc<str>>,
}
```

##### `elbrus-combos`

```rust
// CommanderSpellbook integration
pub struct ComboDatabase {
    // loaded from spellbook JSON / API
    pub combos: Vec<Combo>,
}

pub struct Combo {
    pub id:           Arc<str>,
    pub pieces:       Vec<Uuid>,        // oracle_ids
    pub results:      Vec<Arc<str>>,    // text descriptions
    pub steps:        Option<Arc<str>>,
    pub identity:     ColorSet,
    pub popularity:   Option<u32>,
}

impl ComboDatabase {
    /// Find all combos completable with given card pool.
    pub fn find_enabled_combos(&self, pool: &[Uuid]) -> Vec<&Combo>;

    /// Find combos that need N or fewer additional pieces.
    pub fn find_near_combos(
        &self, pool: &[Uuid], pieces_missing: usize,
    ) -> Vec<(&Combo, Vec<Uuid>)>;
}
```

##### `elbrus-py` (thin)

```rust
// src/lib.rs
use pyo3::prelude::*;
use elbrus_db::SqliteDb;

#[pyclass]
pub struct ElbrusDb(Arc<SqliteDb>);

#[pymethods]
impl ElbrusDb {
    #[staticmethod]
    #[pyo3(name = "open")]
    fn py_open(path: &str) -> PyResult<Self> {
        let rt = tokio::runtime::Runtime::new()?;
        let db = rt.block_on(SqliteDb::open(path)).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(Self(Arc::new(db)))
    }

    fn search_name(&self, query: &str, limit: u32) -> PyResult<Vec<PyObject>> {
        // block_on the async fn, convert results to Python dicts via serde_json intermediary
        todo!()
    }
}
```

Pattern: embed a `tokio::runtime::Runtime` in the `ElbrusDb` struct, `block_on` all async calls. Never hold the GIL across async waits — acquire it only when building the Python return value. Use `serde_json::to_string` → `py.eval("json.loads(...)")` as the bridge for complex types; avoids writing a `ToPyObject` impl for every struct.

---

#### Known Hard Problems

##### 1. Rules Engine Object Model (Phase 5)

The CR zone/stack model doesn't map cleanly to Rust ownership. The core tension:

- A permanent on the battlefield is simultaneously "owned" by a player, "controlled" by another, has triggered abilities "on the stack," and can be targeted by spells that haven't resolved yet — a web of references.
- Rust wants a single owner. You'll need either an entity-component model (ECS: `hecs` or `shipyard`) with `Entity` handles as indices, or a `SlotMap`-based arena with `SlotMap<CardKey, PermanentState>`.
- **Recommendation**: Use `slotmap` arenas for all game objects; pass handles (`ObjectId(u32)`) everywhere. This sidesteps lifetime hell and is WASM-safe.
- The stack is `Vec<StackObject>` where a `StackObject` holds an `ObjectId` plus a snapshot of the object's state at time of casting (copies matter for spells on the stack).

```rust
// Phase 2 placeholder — lock the handle type now
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ZoneId(pub u8);

pub struct GameState {
    pub objects: slotmap::SlotMap<ObjectKey, GameObj>,
    pub zones:   HashMap<ZoneId, Vec<ObjectKey>>,
    pub stack:   Vec<StackEntry>,
    // ...
}
```

Even if Phase 5 is months away, lock `ObjectId` and `ZoneId` as newtypes in `elbrus-core` now, so other crates can reference them without creating a dependency cycle later.

##### 2. WASM SQLite

`sqlx` with SQLite compiles to native (uses libsqlite3). It does **not** work in WASM. Options:

| Approach                          | Ergonomics             | Persistence      |
| --------------------------------- | ---------------------- | ---------------- |
| `wa-sqlite` + OPFS via JS shim    | Medium — JS bridge     | Full, async      |
| `absurd-sql`                      | Low — abandonware risk | Emscripten-heavy |
| IndexedDB directly                | High boilerplate       | No SQL queries   |
| Ship pre-built DB as static asset | Simple, read-only only | Read-only        |

**Recommendation**: For WASM, ship a pre-built snapshot SQLite file as a WASM asset, load it with `wa-sqlite` in OPFS mode, implement `StorageBackend` as a JS-bridged async impl. For read-heavy browser use (search, deck analysis), this is fine. Writes (collection tracking) can sync back to a server or serialize to JSON.

##### 3. PyO3 GIL

- Never store `PyObject` or `Python<'_>` in a Rust struct field.
- All core types must be `Send + Sync` (they are, given `Arc<str>` instead of `Rc`).
- For async: embed one `tokio::Runtime` per `ElbrusDb` Python object, `block_on` at the boundary. Python is single-threaded in the GIL anyway, so this is fine.
- For `__repr__`/`__eq__`: derive `Debug`/`PartialEq` on all types; implement `__repr__` by formatting the `Debug` output — zero extra work.
- Pickle support: implement `__getstate__`/`__setstate__` via `serde_json` serialization on all types that cross the boundary.

##### 4. `OracleText` Parsing

The hard part isn't the happy path — it's edge cases: `{T}: Add {G}.` with a mana activation, `Cycling {2}` (keyword with cost), `Protection from [quality]` (parameterized), saga chapter symbols `I, II, III`, and adventure split-card text. Build a PEG parser (`pest` or `nom`) rather than regex. Start with a simple pass that handles the 90% case and `OracleTextSegment::Text` fallthrough for anything unrecognized — this is correct behavior and won't break as you improve the parser.

##### 5. `f32` vs `f64` for CMC / Probability

- **CMC**: Use `f32`. CMC is always a non-negative multiple of 0.5 (due to {X} and {½}). You could use `Decimal` but it's overkill.
- **Probabilities**: Use `f64`. Hypergeometric calculations with small populations lose precision in `f32`; the Frank Karsten model involves products of many fractions.
- **Prices**: `rust_decimal::Decimal` only. Never float.

##### 6. Schema Migrations and `sqlx::query!`

The compile-time `query!` macro checks against an actual SQLite file at compile time (`DATABASE_URL` env var). This is a CI headache. Strategy:

- Commit `sqlx-data.json` (offline cache via `cargo sqlx prepare`) to the repo.
- In CI: `SQLX_OFFLINE=true cargo build` to skip the live DB check.
- Migration runner at startup: `sqlx::migrate!("./migrations").run(&pool).await?`
- Keep all `query!` macros in `elbrus-db` only — never in other crates.

##### 7. Python 3.14 Compatibility (Temporary)

Due to the very recent release of Python 3.14, `pyo3 v0.23` requires an environment variable to bypass version checks if building against 3.14 before official stability:
`PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 cargo build`

##### 8. Serialization of `Arc<str>`

By default, `serde` does not implement `Serialize`/`Deserialize` for `Arc<T>`. Ensure the `rc` feature is enabled in the workspace `serde` dependency to allow `Arc<str>` to be used as a drop-in for `String` in core types while maintaining `Clone`-cheap efficiency.

##### 9. Bitflags and Serde

When using `bitflags!`, serialize/deserialize support is not automatic even if types inside have it. Always enable the `serde` feature in the `bitflags` dependency to allow `ColorSet` and other bitmask types to round-trip through JSON correctly.

---

#### CI Skeleton

```yaml
### .github/workflows/ci.yml
name: CI
on: [push, pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { targets: "wasm32-unknown-unknown" }
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all -- --check
      - run: cargo clippy --all-targets --all-features -- -D warnings
      - run: cargo test --all-features
        env: { SQLX_OFFLINE: true }
      - run: cargo build -p elbrus-wasm --target wasm32-unknown-unknown --no-default-features
      - run: pip install maturin && maturin build -m crates/elbrus-py/Cargo.toml --no-sdist

  publish:
    if: startsWith(github.ref, 'refs/tags/')
    needs: check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo publish -p elbrus-core
      ### ... ordered by dependency graph
```

---

#### Phase Milestones

**Phase 1 — Foundation** (lock before anything else)

- [x] Workspace skeleton, all crate `Cargo.toml`s with `[workspace.dependencies]` refs
- [x] `elbrus-core` types compile clean with full derive set
- [ ] `elbrus-scryfall` ingests `oracle_cards` bulk file, emits `(OracleCard, Printing)` pairs
- [ ] `elbrus-db` SQLite schema, migrations, `CardRepository` impl, FTS5 on oracle text
- [ ] Ingest a full Scryfall bulk dump in < 60s
- [ ] CI green on fmt + clippy + test

**Phase 2 — User-Facing**

- [ ] `elbrus-deck`: Arena/MTGO/Moxfield parse/serialize, name resolution against db
- [ ] `elbrus-collection`: inventory CRUD, price snapshot ingest, want-list diff
- [ ] `elbrus-rules` Phase 1: CR text parse, keyword registry, format legality + B&R timeline

**Phase 3 — Analysis**

- [ ] `elbrus-analysis`: hypergeometric, mana curve, Frank Karsten, mulligan sim
- [ ] `elbrus-combos`: CommanderSpellbook ingest, `find_enabled_combos`, near-combo detection
- [ ] `elbrus-draft`: booster sim, `PickStrategy` trait, cube draft state

**Phase 4 — Interfaces**

- [ ] `elbrus-cli`: `elbrus search`, `elbrus deck analyze`, `elbrus collection diff`, `elbrus combo check`
- [ ] `elbrus-py`: wheel via maturin, PyPI publish on tag, type stubs (`.pyi`)
- [ ] `elbrus-wasm`: npm package via `wasm-pack`, OPFS storage backend

**Phase 5 — Rules Simulation**

- [ ] Zone model, object lifecycle, stack (APNAP)
- [ ] Triggered/activated ability resolution
- [ ] State-based actions
- [ ] Eventually: deterministic game replay for testing

---

#### One Thing to Do Today

Bootstrap the workspace and get `elbrus-core` compiling with all types above. That's your keystone — every other crate is blocked on it. Two hours of focused work gets you to `cargo test -p elbrus-core` green, and then the rest of the build order flows naturally.
