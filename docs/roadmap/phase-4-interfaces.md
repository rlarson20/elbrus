# Phase 4 — Interfaces (`elbrus-cli` → `elbrus-py` → `elbrus-wasm`)

## 4A. `elbrus-cli` — Command-Line Interface

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

## 4B. `elbrus-py` — Python Bindings

- [ ] Embed `tokio::Runtime` in a `#[pyclass] ElbrusDb` struct
- [ ] Core methods (all `block_on` at boundary, release GIL during async):
  - [ ] `ElbrusDb.open(path: str) → ElbrusDb`
  - [ ] `ElbrusDb.search_name(query: str, limit: int) → list[dict]`
  - [ ] `ElbrusDb.search_text(query: str, limit: int) → list[dict]`
  - [ ] `ElbrusDb.get_card(uuid: str) → dict`
  - [ ] `ElbrusDb.ingest(path: str) → dict` (IngestStats)
- [ ] Bridge types via `serde_json → json.loads()` (avoid per-type `ToPyObject`)
- [ ] `__repr__` / `__eq__` via `Debug` / `PartialEq` derives
- [ ] Python serialization support via `__getstate__`/`__setstate__` with `serde_json`
- [ ] Type stubs (`.pyi` file) for IDE autocomplete
- [ ] `maturin` build config: `pyproject.toml` or `Cargo.toml` metadata
- [ ] Test: `pip install -e .` then `import elbrus` smoke test

## 4C. `elbrus-wasm` — Browser Bindings

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
