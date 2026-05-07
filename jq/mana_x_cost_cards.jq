[.[] | select(.mana_cost? | strings | contains("X")) | {name, mana_cost}]
