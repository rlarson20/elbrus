[.[] | select(.mana_cost? | strings | contains("/P")) | {name, mana_cost}]
