# Phase 5 — Rules Simulation (`elbrus-rules` Phase 2)

## 5A. Game Object Model

- [ ] Define zone model
  - [ ] `ZoneId(u8)` enum: Library, Hand, Battlefield, Graveyard, Stack, Exile, Command
  - [ ] `GameState { objects: SlotMap<ObjectKey, GameObj>, zones: HashMap<ZoneId, Vec<ObjectKey>>, stack: Vec<StackEntry> }`
- [ ] Define `GameObj` — unified object for cards/tokens/copies on stack
  - [ ] Current characteristics (name, types, P/T, abilities — may differ from printed)
  - [ ] Controller, owner, timestamps, counters
- [ ] `StackEntry` — object + snapshot of state at time of casting
- [ ] Zone transfer API: `move_object(obj, from_zone, to_zone) → Result`

## 5B. Core Game Actions

- [ ] Turn structure state machine (untap → upkeep → draw → main1 → combat → main2 → end)
- [ ] Priority system (APNAP order)
- [ ] Casting spells: move to stack, pay costs, resolve
- [ ] Activated abilities: pay cost, put on stack, resolve
- [ ] Triggered abilities: trigger → go on stack in APNAP order

## 5C. State-Based Actions

- [ ] Creature dies (toughness ≤ 0 or lethal damage)
- [ ] Player loses (life ≤ 0, can't draw, poison ≥ 10)
- [ ] Legend rule, planeswalker uniqueness
- [ ] Token removal from non-battlefield zones
- [ ] +1/+1 and -1/-1 counter annihilation

## 5D. Testing & Determinism

- [ ] Deterministic game replay: seed RNG, record all choices → replay produces identical state
- [ ] Golden-file tests: known game sequences produce known final states
- [ ] Fuzz testing: random valid action sequences don't panic
