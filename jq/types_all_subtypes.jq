[.[] | .type_line? | strings | split(" — ") | select(length > 1) | .[1] | split(" ")[]] | unique | sort
