# Phase 2 — User-Facing (`elbrus-deck` → `elbrus-collection` → `elbrus-rules` P1)

## 2A. `elbrus-deck` — Deck Parsing

- [ ] Implement `ArenaParser`
  - [ ] `can_parse()` — detect Arena export format (e.g., `Deck\n` header or `N CardName` lines)
  - [ ] `parse()` — parse `N CardName (SET) CN` lines into `Deck`
  - [ ] `serialize()` — emit Arena-format text
  - [ ] Handle Commander/Companion sections
- [ ] Implement `MtgoParser`
  - [ ] Parse MTGO `.dek` XML format or text export
  - [ ] Handle sideboard markers
- [ ] Implement `MoxfieldParser`
  - [ ] Parse Moxfield CSV/text export format
  - [ ] Handle Moxfield-specific sections (considering, maybeboard)
- [ ] Add `DeckResolver` — resolve `card_name` → `Uuid` via `CardRepository` lookup
  - [ ] Fuzzy matching for minor name variations
  - [ ] Set hint resolution (prefer matching set code)
- [ ] Deck validation
  - [ ] Format-specific validation: minimum deck size, max copies, banned cards
  - [ ] Commander-specific: color identity check, singleton rule
- [ ] Unit tests for each parser format
- [ ] Round-trip tests: parse → serialize → parse = same `Deck`

## 2B. `elbrus-collection` — Inventory Management

- [ ] Define collection types
  - [ ] `CollectionEntry { printing_id: Uuid, quantity: u32, condition: Condition, foil: bool, notes: Option<Arc<str>> }`
  - [ ] `enum Condition { NearMint, LightlyPlayed, ModeratelyPlayed, HeavyPlayed, Damaged }`
  - [ ] `Collection { entries: Vec<CollectionEntry> }`
- [ ] Implement `CollectionRepository` trait in `elbrus-db`
  - [ ] CRUD: add, update quantity, remove, list
  - [ ] Bulk import from CSV
- [ ] Price snapshot ingest from Scryfall bulk data
  - [ ] Store historical snapshots with `fetched_at` timestamp
  - [ ] Query: current value, price history, total collection value
- [ ] Want-list diffing
  - [ ] `WantList { entries: Vec<WantEntry> }` — cards you want to acquire
  - [ ] `diff(collection, want_list) → Vec<MissingCard>` — what you still need
- [ ] Unit tests + integration tests with in-memory db

## 2C. `elbrus-rules` Phase 1 — Data Only

- [ ] Define types: `ComprehensiveRules`, `RuleNumber`, `Rule`, `KeywordRegistry`, `KeywordDefinition`
- [ ] Implement CR text parser
  - [ ] Parse the Comprehensive Rules plaintext file (from WotC)
  - [ ] Build `BTreeMap<RuleNumber, Rule>` from numbered sections
  - [ ] Extract glossary from the glossary section
  - [ ] Cross-reference `see_also` links between rules
- [ ] Build `KeywordRegistry` from CR section 702 (abilities) and 701 (actions)
  - [ ] Populate `takes_cost`, `is_evasion`, summary for each keyword
- [ ] Format legality + B&R timeline
  - [ ] Define `BanList`, `BanEntry` types
  - [ ] Data source: scrape or manually maintain B&R data
  - [ ] Query: "was card X legal in format Y on date Z?"
- [ ] Unit tests for CR parsing, keyword registry population
