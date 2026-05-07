group_by(.layout) | map({layout: .[0].layout, has_faces: (map(select(.card_faces != null)) | length), total: length})
