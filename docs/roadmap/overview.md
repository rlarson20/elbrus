# Overview

## TL;DR

- **5 phases**, gated on type stability; Phase 1 is the investment that makes everything else cheap
- Lock `elbrus-core` types _before_ writing a single `sqlx::query!` macro — compile-time query checks bite you if the schema drifts
- WASM SQLite is the gnarliest constraint; design `elbrus-db` with a pluggable storage trait so WASM can swap to `absurd-sql`/OPFS without touching business logic
- PyO3 GIL: keep bindings `Send + Sync` by making all core types `Clone`-cheap; never hold Python objects in Rust structs
- `OracleText` as `Vec<OracleTextSegment>` from day one is the right call — retrofitting this is painful

## Phase Map

```
Phase 1 (Foundation)     core → scryfall → db
Phase 2 (User-Facing)    deck → collection → rules-P1
Phase 3 (Analysis)       analysis → combos → draft
Phase 4 (Interface)      cli → py → wasm
Phase 5 (Rules Sim)      rules-P2 (zone/stack model)
```

## Critical Early Decisions (Lock These First)

| Decision         | Choice                                                                         | Rationale                                              |
| ---------------- | ------------------------------------------------------------------------------ | ------------------------------------------------------ |
| Primary key      | Scryfall UUID (`uuid::Uuid`)                                                   | Stable, content-addressed                              |
| Oracle text repr | `Vec<OracleTextSegment>`                                                       | Enables rules parsing, FTS, diff                       |
| Async runtime    | `tokio` workspace-wide                                                         | sqlx async, consistent executor                        |
| Error strategy   | `thiserror` per-crate, `anyhow` at binary/binding layer                        | Structured errors propagate cleanly                    |
| Serde roundtrip  | All core types must `serde_json::from_str(serde_json::to_string(&v)) == Ok(v)` | Required for WASM postMessage and Python serialization |
| WASM storage     | Abstract `StorageBackend` trait in `elbrus-db`                                 | OPFS/absurd-sql swap without API break                 |
| Keyword registry | `#[non_exhaustive] enum Keyword` + `UnknownKeyword(Arc<str>)` variant          | CR adds keywords; don't break consumers                |
| Feature flags    | `fts` on db, `sim` on rules                                                    | Keep WASM bundle minimal                               |

## Phase Milestones (success criteria, not implementation steps)

**Phase 1 — Foundation**

- [x] Workspace skeleton, all crate `Cargo.toml`s with `[workspace.dependencies]` refs
- [x] `elbrus-core` types compile clean with full derive set
- [x] `elbrus-scryfall` ingests `oracle_cards` bulk file, emits `(OracleCard, Printing)` pairs
- [x] `elbrus-db` SQLite schema, migrations, `CardRepository` impl, FTS5 on oracle text
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

For implementation steps, see the per-phase files.
