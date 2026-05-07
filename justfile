# justfile

set shell := ["bash", "-cu"]

bulk := "data/json/oracle-cards-20260214220258.json"
outdir := "data/jq_exports"

# Ensure output directory exists

prepare:
	mkdir -p {{outdir}}

manacosts-sqlite:
	sqlite3 elbrus.db "SELECT DISTINCT mana_cost FROM card_faces WHERE mana_cost IS NOT NULL" | sort -u > /tmp/mana_costs.txt && cat /tmp/mana_costs.txt


# ---------- Orientation ----------

orientation-count: prepare
	jq 'length' {{bulk}} > {{outdir}}/orientation_card_count.json

orientation-keys: prepare
	jq '.[0] | keys' {{bulk}} > {{outdir}}/orientation_top_level_keys.json

orientation-layouts: prepare
	jq '[.[].layout] | unique | sort' {{bulk}} > {{outdir}}/orientation_layouts.json

orientation-rarities: prepare
	jq '[.[].rarity] | unique' {{bulk}} > {{outdir}}/orientation_rarities.json

# ---------- Mana cost corpus ----------

mana-distinct-root: prepare
	jq '[.[].mana_cost // empty] | unique | sort' {{bulk}} > {{outdir}}/mana_distinct_costs_root.json

mana-distinct-all: prepare
	jq '[.. | objects | .mana_cost? // empty] | unique | sort' {{bulk}} > {{outdir}}/mana_distinct_costs_all_faces.json

mana-no-cost: prepare
	jq '[.[] | select(.mana_cost == null or .mana_cost == "") | .name]' {{bulk}} > {{outdir}}/mana_no_cost_cards.json

mana-x: prepare
	jq '[.[] | select(.mana_cost? | strings | contains("X")) | {name, mana_cost}]' {{bulk}} > {{outdir}}/mana_x_cost_cards.json

mana-hybrid: prepare
	jq '[.[] | select(.mana_cost? | strings | test("\\{[0-9WUBRG]/[WUBRG]\\}")) | {name, mana_cost}]' {{bulk}} > {{outdir}}/mana_hybrid_cards.json

mana-phyrexian: prepare
	jq '[.[] | select(.mana_cost? | strings | contains("/P")) | {name, mana_cost}]' {{bulk}} > {{outdir}}/mana_phyrexian_cards.json

# ---------- Type line corpus ----------

types-distinct: prepare
	jq '[.[].type_line // empty] | unique | sort' {{bulk}} > {{outdir}}/types_distinct_type_lines.json

types-no-subtypes: prepare
	jq '[.[] | select(.type_line? | strings | contains("—") | not) | {name, type_line}]' {{bulk}} > {{outdir}}/types_no_subtypes.json

types-subtypes: prepare
	jq '[.[] | .type_line? | strings | split(" — ") | select(length > 1) | .[1] | split(" ")[]] | unique | sort' {{bulk}} > {{outdir}}/types_all_subtypes.json

types-multitype: prepare
	jq '[.[] | select((.type_line? | strings | split(" — ")[0] | split(" ") | length) > 2) | {name, type_line}]' {{bulk}} > {{outdir}}/types_multitype_cards.json

# ---------- Layout / multi-face ----------

layout-faces: prepare
	jq '[.[] | select(.card_faces != null) | {name, layout, faces: [.card_faces[].name]}]' {{bulk}} > {{outdir}}/layout_cards_with_faces.json

layout-face-usage: prepare
	jq 'group_by(.layout) | map({layout: .[0].layout, has_faces: (map(select(.card_faces != null)) | length), total: length})' {{bulk}} > {{outdir}}/layout_face_usage_by_layout.json

# ---------- Legalities ----------

legalities-formats: prepare
	jq '.[0].legalities | keys' {{bulk}} > {{outdir}}/legalities_formats.json

legalities-commander: prepare
	jq '[.[] | select(.legalities.commander == "legal") | .name]' {{bulk}} > {{outdir}}/legalities_commander_legal_cards.json

legalities-banned: prepare
	jq '[.[] | . as $c | .legalities | to_entries[] | select(.value == "banned") | {card: $c.name, format: .key}]' {{bulk}} > {{outdir}}/legalities_banned_cards.json

# ---------- Prices ----------

prices-null: prepare
	jq '[.[] | select(.prices.usd == null) | {name, digital, set}]' {{bulk}} > {{outdir}}/prices_null_usd_cards.json

prices-distribution: prepare
	jq '[.[] | .prices.usd | select(. != null) | tonumber] | {min: min, max: max, avg: (add/length)}' {{bulk}} > {{outdir}}/prices_distribution.json

# ---------- Digital cards ----------

digital-only: prepare
	jq '[.[] | select(.digital == true) | {name, set, games}]' {{bulk}} > {{outdir}}/digital_only_cards.json

digital-games: prepare
	jq '[.[].games[]] | group_by(.) | map({game: .[0], count: length})' {{bulk}} > {{outdir}}/digital_games_distribution.json

# ---------- Run everything ----------

all:
	just orientation-count
	just orientation-keys
	just orientation-layouts
	just orientation-rarities
	just mana-distinct-root
	just mana-distinct-all
	just mana-no-cost
	just mana-x
	just mana-hybrid
	just mana-phyrexian
	just types-distinct
	just types-no-subtypes
	just types-subtypes
	just types-multitype
	just layout-faces
	just layout-face-usage
	just legalities-formats
	just legalities-commander
	just legalities-banned
	just prices-null
	just prices-distribution
	just digital-only
	just digital-games
