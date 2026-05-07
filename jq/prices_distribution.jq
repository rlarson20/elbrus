[.[] | .prices.usd | select(. != null) | tonumber] | {min: min, max: max, avg: (add/length)}
