[.[] | select(.type_line? | strings | contains("—") | not) | {name, type_line}]
