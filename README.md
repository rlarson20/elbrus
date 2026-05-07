# elbrus

A Rust workspace of Magic: The Gathering tools — card data, deck building,
collection tracking, and analysis. Cargo workspace plus Python (`pyo3`) and
WebAssembly (`wasm-bindgen`) bindings.

## Crates

Active:

- `elbrus-core` — domain types: cards, mana, colors, types, legalities, oracle, rulings.
- `elbrus-scryfall` — Scryfall bulk data ingestion and conversion to core types.
- `elbrus-db` — SQLite storage layer with migrations and FTS5.
- `elbrus-deck` — deck representation.
- `elbrus-parser` — parsers (mana costs, etc.) built on `nom`.
- `elbrus-cli` — `elbrus` binary.
- `elbrus-py` — Python extension module.
- `elbrus-wasm` — WebAssembly bindings.

Stubs (commented out in `Cargo.toml` until they have content):
`elbrus-rules`, `elbrus-draft`, `elbrus-collection`, `elbrus-analysis`,
`elbrus-combos`.

## Development

```bash
cargo check
cargo test
```

## Data exploration

Scryfall bulk JSON dumps go in `data/json/` (gitignored). The `justfile` runs
`jq` programs from `jq/` against the most recent `oracle-cards-*.json` and
writes results into `data/jq_exports/`.

```bash
just all                  # run every analysis query
just mana-hybrid          # run a single one
```

Each recipe maps to a `jq/<name>.jq` file with the same basename as its output.

## Layout

```
crates/        Rust crates
jq/            jq programs used by the justfile
docs/          roadmap and seed-conversation notes
scripts/       Python helpers for fetching/sampling Scryfall data
data/          local-only: bulk JSON, exports, sqlite db (gitignored)
```

## License

Dual-licensed under MIT or Apache-2.0.
