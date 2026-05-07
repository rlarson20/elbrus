# justfile

set shell := ["bash", "-cu"]

bulk := `ls -t data/json/oracle-cards-*.json 2>/dev/null | head -1`
outdir := "data/jq_exports"
jqdir := "jq"

# Ensure output directory exists

prepare:
	mkdir -p {{outdir}}

manacosts-sqlite:
	sqlite3 elbrus.db "SELECT DISTINCT mana_cost FROM card_faces WHERE mana_cost IS NOT NULL" | sort -u > /tmp/mana_costs.txt && cat /tmp/mana_costs.txt


# ---------- Orientation ----------

orientation-count: prepare
	jq -f {{jqdir}}/orientation_card_count.jq {{bulk}} > {{outdir}}/orientation_card_count.json

orientation-keys: prepare
	jq -f {{jqdir}}/orientation_top_level_keys.jq {{bulk}} > {{outdir}}/orientation_top_level_keys.json

orientation-layouts: prepare
	jq -f {{jqdir}}/orientation_layouts.jq {{bulk}} > {{outdir}}/orientation_layouts.json

orientation-rarities: prepare
	jq -f {{jqdir}}/orientation_rarities.jq {{bulk}} > {{outdir}}/orientation_rarities.json

# ---------- Mana cost corpus ----------

mana-distinct-root: prepare
	jq -f {{jqdir}}/mana_distinct_costs_root.jq {{bulk}} > {{outdir}}/mana_distinct_costs_root.json

mana-distinct-all: prepare
	jq -f {{jqdir}}/mana_distinct_costs_all_faces.jq {{bulk}} > {{outdir}}/mana_distinct_costs_all_faces.json

mana-no-cost: prepare
	jq -f {{jqdir}}/mana_no_cost_cards.jq {{bulk}} > {{outdir}}/mana_no_cost_cards.json

mana-x: prepare
	jq -f {{jqdir}}/mana_x_cost_cards.jq {{bulk}} > {{outdir}}/mana_x_cost_cards.json

mana-hybrid: prepare
	jq -f {{jqdir}}/mana_hybrid_cards.jq {{bulk}} > {{outdir}}/mana_hybrid_cards.json

mana-phyrexian: prepare
	jq -f {{jqdir}}/mana_phyrexian_cards.jq {{bulk}} > {{outdir}}/mana_phyrexian_cards.json

# ---------- Type line corpus ----------

types-distinct: prepare
	jq -f {{jqdir}}/types_distinct_type_lines.jq {{bulk}} > {{outdir}}/types_distinct_type_lines.json

types-no-subtypes: prepare
	jq -f {{jqdir}}/types_no_subtypes.jq {{bulk}} > {{outdir}}/types_no_subtypes.json

types-subtypes: prepare
	jq -f {{jqdir}}/types_all_subtypes.jq {{bulk}} > {{outdir}}/types_all_subtypes.json

types-multitype: prepare
	jq -f {{jqdir}}/types_multitype_cards.jq {{bulk}} > {{outdir}}/types_multitype_cards.json

# ---------- Layout / multi-face ----------

layout-faces: prepare
	jq -f {{jqdir}}/layout_cards_with_faces.jq {{bulk}} > {{outdir}}/layout_cards_with_faces.json

layout-face-usage: prepare
	jq -f {{jqdir}}/layout_face_usage_by_layout.jq {{bulk}} > {{outdir}}/layout_face_usage_by_layout.json

# ---------- Legalities ----------

legalities-formats: prepare
	jq -f {{jqdir}}/legalities_formats.jq {{bulk}} > {{outdir}}/legalities_formats.json

legalities-commander: prepare
	jq -f {{jqdir}}/legalities_commander_legal_cards.jq {{bulk}} > {{outdir}}/legalities_commander_legal_cards.json

legalities-banned: prepare
	jq -f {{jqdir}}/legalities_banned_cards.jq {{bulk}} > {{outdir}}/legalities_banned_cards.json

# ---------- Prices ----------

prices-null: prepare
	jq -f {{jqdir}}/prices_null_usd_cards.jq {{bulk}} > {{outdir}}/prices_null_usd_cards.json

prices-distribution: prepare
	jq -f {{jqdir}}/prices_distribution.jq {{bulk}} > {{outdir}}/prices_distribution.json

# ---------- Digital cards ----------

digital-only: prepare
	jq -f {{jqdir}}/digital_only_cards.jq {{bulk}} > {{outdir}}/digital_only_cards.json

digital-games: prepare
	jq -f {{jqdir}}/digital_games_distribution.jq {{bulk}} > {{outdir}}/digital_games_distribution.json

# ---------- Run everything ----------

all: orientation-count orientation-keys orientation-layouts orientation-rarities \
     mana-distinct-root mana-distinct-all mana-no-cost mana-x mana-hybrid mana-phyrexian \
     types-distinct types-no-subtypes types-subtypes types-multitype \
     layout-faces layout-face-usage \
     legalities-formats legalities-commander legalities-banned \
     prices-null prices-distribution \
     digital-only digital-games
