CREATE TABLE collections (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT
);

CREATE TABLE collection_entries (
    collection_id TEXT NOT NULL,
    printing_id TEXT NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 1,
    condition TEXT NOT NULL,
    foil BOOLEAN NOT NULL DEFAULT FALSE,
    notes TEXT,
    PRIMARY KEY (collection_id, printing_id, condition, foil),
    FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE,
    FOREIGN KEY (printing_id) REFERENCES printings(id) ON DELETE CASCADE
);

CREATE INDEX idx_collection_entries_printing_id ON collection_entries(printing_id);

CREATE TABLE price_snapshots (
    printing_id TEXT NOT NULL,
    fetched_at TEXT NOT NULL,
    usd TEXT,
    usd_foil TEXT,
    eur TEXT,
    tix TEXT,
    PRIMARY KEY (printing_id, fetched_at),
    FOREIGN KEY (printing_id) REFERENCES printings(id) ON DELETE CASCADE
);
