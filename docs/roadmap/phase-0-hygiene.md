# Phase 0 — Repository Hygiene

> Living checklist. Mark items: `[ ]` not started · `[/]` in progress · `[x]` done

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
