# Known Hard Problems

## 1. Rules Engine Object Model (Phase 5)

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

## 2. WASM SQLite

`sqlx` with SQLite compiles to native (uses libsqlite3). It does **not** work in WASM. Options:

| Approach                          | Ergonomics             | Persistence      |
| --------------------------------- | ---------------------- | ---------------- |
| `wa-sqlite` + OPFS via JS shim    | Medium — JS bridge     | Full, async      |
| `absurd-sql`                      | Low — abandonware risk | Emscripten-heavy |
| IndexedDB directly                | High boilerplate       | No SQL queries   |
| Ship pre-built DB as static asset | Simple, read-only only | Read-only        |

**Recommendation**: For WASM, ship a pre-built snapshot SQLite file as a WASM asset, load it with `wa-sqlite` in OPFS mode, implement `StorageBackend` as a JS-bridged async impl. For read-heavy browser use (search, deck analysis), this is fine. Writes (collection tracking) can sync back to a server or serialize to JSON.

## 3. PyO3 GIL

- Never store `PyObject` or `Python<'_>` in a Rust struct field.
- All core types must be `Send + Sync` (they are, given `Arc<str>` instead of `Rc`).
- For async: embed one `tokio::Runtime` per `ElbrusDb` Python object, `block_on` at the boundary. Python is single-threaded in the GIL anyway, so this is fine.
- For `__repr__`/`__eq__`: derive `Debug`/`PartialEq` on all types; implement `__repr__` by formatting the `Debug` output — zero extra work.
- Python serialization support: implement `__getstate__`/`__setstate__` via `serde_json` on all types that cross the boundary.

## 4. `OracleText` Parsing

The hard part isn't the happy path — it's edge cases: `{T}: Add {G}.` with a mana activation, `Cycling {2}` (keyword with cost), `Protection from [quality]` (parameterized), saga chapter symbols `I, II, III`, and adventure split-card text. Build a PEG parser (`pest` or `nom`) rather than regex. Start with a simple pass that handles the 90% case and `OracleTextSegment::Text` fallthrough for anything unrecognized — this is correct behavior and won't break as you improve the parser.

## 5. `f32` vs `f64` for CMC / Probability

- **CMC**: Use `f32`. CMC is always a non-negative multiple of 0.5 (due to {X} and {½}). You could use `Decimal` but it's overkill.
- **Probabilities**: Use `f64`. Hypergeometric calculations with small populations lose precision in `f32`; the Frank Karsten model involves products of many fractions.
- **Prices**: `rust_decimal::Decimal` only. Never float.

## 6. Schema Migrations and `sqlx::query!`

The compile-time `query!` macro checks against an actual SQLite file at compile time (`DATABASE_URL` env var). This is a CI headache. Strategy:

- Commit `sqlx-data.json` (offline cache via `cargo sqlx prepare`) to the repo.
- In CI: `SQLX_OFFLINE=true cargo build` to skip the live DB check.
- Migration runner at startup: `sqlx::migrate!("./migrations").run(&pool).await?`
- Keep all `query!` macros in `elbrus-db` only — never in other crates.

## 7. Serialization of `Arc<str>`

By default, `serde` does not implement `Serialize`/`Deserialize` for `Arc<T>`. Ensure the `rc` feature is enabled in the workspace `serde` dependency to allow `Arc<str>` to be used as a drop-in for `String` in core types while maintaining `Clone`-cheap efficiency.

## 8. Bitflags and Serde

When using `bitflags!`, serialize/deserialize support is not automatic even if types inside have it. Always enable the `serde` feature in the `bitflags` dependency to allow `ColorSet` and other bitmask types to round-trip through JSON correctly.
