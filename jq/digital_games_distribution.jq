[.[].games[]] | group_by(.) | map({game: .[0], count: length})
