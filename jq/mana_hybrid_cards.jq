[.[] | select(.mana_cost? | strings | test("\\{[0-9WUBRG]/[WUBRG]\\}")) | {name, mana_cost}]
