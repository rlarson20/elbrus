[.[] | select((.type_line? | strings | split(" — ")[0] | split(" ") | length) > 2) | {name, type_line}]
