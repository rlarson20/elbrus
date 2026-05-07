# Phase 3 — Analysis (`elbrus-analysis` → `elbrus-combos` → `elbrus-draft`)

## 3A. `elbrus-analysis` — Statistical Tools

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

## 3B. `elbrus-combos` — Commander Spellbook Integration

- [ ] Define `ComboDatabase`, `Combo` types
- [ ] Ingest Commander Spellbook data (JSON API or bulk export)
  - [ ] Map card names → oracle_ids via db lookup
  - [ ] Store combos with pieces, results, steps, color identity
- [ ] `find_enabled_combos(pool) → Vec<&Combo>` — all combos possible with given cards
- [ ] `find_near_combos(pool, pieces_missing) → Vec<(&Combo, Vec<Uuid>)>` — combos needing ≤ N more pieces
- [ ] Integration tests with sample combo data

## 3C. `elbrus-draft` — Draft Simulation

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
