# API Sketches (Unbuilt Crates)

> **Source of truth is the actual code.** For crates that already exist
> (`elbrus-core`, `elbrus-scryfall`, `elbrus-db`, `elbrus-parser`, `elbrus-cli`),
> the implementation in `crates/<name>/src/` supersedes any sketch here. These
> sketches are kept as design intent for crates that are still empty stubs:
> `elbrus-deck` (skeleton only), `elbrus-collection`, `elbrus-analysis`,
> `elbrus-rules`, `elbrus-combos`, `elbrus-py` (skeleton), `elbrus-wasm` (skeleton).

## `elbrus-deck`

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

## `elbrus-analysis`

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

## `elbrus-rules`

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

## `elbrus-combos`

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

## `elbrus-py` (thin)

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

Pattern: embed a `tokio::runtime::Runtime` in the `ElbrusDb` struct, `block_on` all async calls. Never hold the GIL across async waits — acquire it only when building the Python return value. For complex types, serialize Rust → JSON via `serde_json::to_string`, then deserialize on the Python side via `json.loads` (call into the `json` module from PyO3); this avoids writing a `ToPyObject` impl for every struct.
