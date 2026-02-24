CREATE TABLE oracle_cards (
    oracle_id TEXT PRIMARY KEY,
    layout TEXT NOT NULL,
    color_identity INTEGER NOT NULL,
    keywords TEXT NOT NULL, -- JSON array
    legalities TEXT NOT NULL, -- JSON object
    edh_rank INTEGER,
    reserved BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE card_faces (
    oracle_id TEXT NOT NULL,
    face_index INTEGER NOT NULL,
    name TEXT NOT NULL,
    mana_cost TEXT,
    type_line TEXT,
    oracle_text TEXT,
    colors INTEGER,
    power TEXT,
    toughness TEXT,
    loyalty TEXT,
    defense TEXT,
    flavor_text TEXT,
    PRIMARY KEY (oracle_id, face_index),
    FOREIGN KEY (oracle_id) REFERENCES oracle_cards(oracle_id) ON DELETE CASCADE
);

CREATE TABLE printings (
    id TEXT PRIMARY KEY,
    oracle_id TEXT NOT NULL,
    set_code TEXT NOT NULL,
    collector_number TEXT NOT NULL,
    rarity TEXT NOT NULL,
    lang TEXT NOT NULL,
    released_at TEXT NOT NULL,
    image_uris TEXT, -- JSON object
    promo BOOLEAN NOT NULL DEFAULT FALSE,
    digital BOOLEAN NOT NULL DEFAULT FALSE,
    full_art BOOLEAN NOT NULL DEFAULT FALSE,
    textless BOOLEAN NOT NULL DEFAULT FALSE,
    reprint BOOLEAN NOT NULL DEFAULT FALSE,
    prices TEXT, -- JSON object
    FOREIGN KEY (oracle_id) REFERENCES oracle_cards(oracle_id) ON DELETE CASCADE
);

-- Indexes (oracle_cards.oracle_id and printings.id are already indexed as PRIMARY KEY)
CREATE INDEX idx_printings_oracle_id ON printings(oracle_id);
CREATE INDEX idx_printings_set_code ON printings(set_code);
CREATE INDEX idx_card_faces_name ON card_faces(name);
