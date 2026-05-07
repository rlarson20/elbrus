The sample is a top-level array of card objects. All queries assume `sample.json` but swap in your bulk file.

---

## Orientation

```bash
# How many cards
jq 'length' oracle-cards-20260214220258.json \
  > data/jq_exports/orientation_card_count.json

# What top-level keys exist
jq '.[0] | keys' oracle-cards-20260214220258.json \
  > data/jq_exports/orientation_top_level_keys.json

# All layouts present
jq '[.[].layout] | unique | sort' oracle-cards-20260214220258.json \
  > data/jq_exports/orientation_layouts.json

# All rarities
jq '[.[].rarity] | unique' oracle-cards-20260214220258.json \
  > data/jq_exports/orientation_rarities.json
```

---

## Mana cost corpus

```bash
# Every distinct mana_cost string
jq '[.[].mana_cost // empty] | unique | sort' oracle-cards-20260214220258.json \
  > data/jq_exports/mana_distinct_costs_root.json

# From card faces too (MDFCs, splits, etc.)
jq '[.. | objects | .mana_cost? // empty] | unique | sort' oracle-cards-20260214220258.json \
  > data/jq_exports/mana_distinct_costs_all_faces.json

# Cards with no mana cost (doesn't handle art_series)
jq '[.[] | select(.mana_cost == null or .mana_cost == "") | .name]' oracle-cards-20260214220258.json \
  > data/jq_exports/mana_no_cost_cards.json

# Cards with X in cost
jq '[.[] | select(.mana_cost? | strings | contains("X")) | {name, mana_cost}]' oracle-cards-20260214220258.json \
  > data/jq_exports/mana_x_cost_cards.json

# Cards with hybrid symbols
jq '[.[] | select(.mana_cost? | strings | test("\\{[0-9WUBRG]/[WUBRG]\\}")) | {name, mana_cost}]' oracle-cards-20260214220258.json \
  > data/jq_exports/mana_hybrid_cards.json

# Cards with phyrexian mana
jq '[.[] | select(.mana_cost? | strings | contains("/P")) | {name, mana_cost}]' oracle-cards-20260214220258.json \
  > data/jq_exports/mana_phyrexian_cards.json
```

---

## Type line corpus

```bash
# Every distinct type_line
jq '[.[].type_line // empty] | unique | sort' oracle-cards-20260214220258.json \
  > data/jq_exports/types_distinct_type_lines.json

# Cards with no em-dash
jq '[.[] | select(.type_line? | strings | contains("—") | not) | {name, type_line}]' oracle-cards-20260214220258.json \
  > data/jq_exports/types_no_subtypes.json

# All unique subtypes
jq '[.[] | .type_line? | strings | split(" — ") | select(length > 1) | .[1] | split(" ")[]] | unique | sort' oracle-cards-20260214220258.json \
  > data/jq_exports/types_all_subtypes.json

# Multi-type cards
jq '[.[] | select((.type_line? | strings | split(" — ")[0] | split(" ") | length) > 2) | {name, type_line}]' oracle-cards-20260214220258.json \
  > data/jq_exports/types_multitype_cards.json
```

---

## Layout / multi-face cards

```bash
# Cards that have card_faces
jq '[.[] | select(.card_faces != null) | {name, layout, faces: [.card_faces[].name]}]' oracle-cards-20260214220258.json \
  > data/jq_exports/layout_cards_with_faces.json

# Layouts that use card_faces vs root-level fields
jq 'group_by(.layout) | map({layout: .[0].layout, has_faces: (map(select(.card_faces != null)) | length), total: length})' oracle-cards-20260214220258.json \
  > data/jq_exports/layout_face_usage_by_layout.json
```

---

## Legalities

```bash
# All format names
jq '.[0].legalities | keys' oracle-cards-20260214220258.json \
  > data/jq_exports/legalities_formats.json

# Cards legal in commander
jq '[.[] | select(.legalities.commander == "legal") | .name]' oracle-cards-20260214220258.json \
  > data/jq_exports/legalities_commander_legal_cards.json

# Cards banned anywhere
jq '[.[] | . as $c | .legalities | to_entries[] | select(.value == "banned") | {card: $c.name, format: .key}]' oracle-cards-20260214220258.json \
  > data/jq_exports/legalities_banned_cards.json
```

---

## Prices / nulls

```bash
# Cards with null USD price
jq '[.[] | select(.prices.usd == null) | {name, digital, set}]' oracle-cards-20260214220258.json \
  > data/jq_exports/prices_null_usd_cards.json

# Price distribution
jq '[.[] | .prices.usd | select(. != null) | tonumber] | {min: min, max: max, avg: (add/length)}' oracle-cards-20260214220258.json \
  > data/jq_exports/prices_distribution.json
```

---

## Digital / Arena-only cards (like the Alchemy cards in your sample)

```bash
# Digital-only cards (by printing, some printings are digital only)
jq '[.[] | select(.digital == true) | {name, set, games}]' oracle-cards-20260214220258.json \
  > data/jq_exports/digital_only_cards.json

# Games field distribution
jq '[.[].games[]] | group_by(.) | map({game: .[0], count: length})' oracle-cards-20260214220258.json \
  > data/jq_exports/digital_games_distribution.json
```
