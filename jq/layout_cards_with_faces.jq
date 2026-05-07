[.[] | select(.card_faces != null) | {name, layout, faces: [.card_faces[].name]}]
