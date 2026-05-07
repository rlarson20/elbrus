[.[] | . as $c | .legalities | to_entries[] | select(.value == "banned") | {card: $c.name, format: .key}]
