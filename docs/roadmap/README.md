# Roadmap

Living docs for elbrus implementation. Mark items: `[ ]` not started · `[/]` in progress · `[x]` done.

## Where to start

- New here? Read [`overview.md`](overview.md) — TL;DR, phase map, locked early decisions, milestone summary.
- Want to know what to do next? Open the lowest-numbered phase file with unchecked items.
- Designing a new crate? Skim [`api-sketches.md`](api-sketches.md) for design intent on the unbuilt ones.
- Hitting a thorny problem? Check [`hard-problems.md`](hard-problems.md) — the gotchas were already mapped.

## Reference

- [`overview.md`](overview.md) — TL;DR, phase map, critical early decisions, milestone summary
- [`hard-problems.md`](hard-problems.md) — 8 known hard problems (rules engine, WASM SQLite, PyO3 GIL, etc.)
- [`api-sketches.md`](api-sketches.md) — design sketches for unbuilt crates (actual code is source of truth where it exists)

## Phases

- [`phase-0-hygiene.md`](phase-0-hygiene.md) — repo hygiene, CI scaffolding
- [`phase-1-foundation.md`](phase-1-foundation.md) — db, scryfall ingest, core parsers
- [`phase-2-user-facing.md`](phase-2-user-facing.md) — deck, collection, rules data
- [`phase-3-analysis.md`](phase-3-analysis.md) — statistics, combos, draft sim
- [`phase-4-interfaces.md`](phase-4-interfaces.md) — cli, python, wasm
- [`phase-5-rules.md`](phase-5-rules.md) — rules simulation engine
- [`stretch.md`](stretch.md) — unscheduled stretch goals
