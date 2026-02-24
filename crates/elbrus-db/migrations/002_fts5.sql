-- 002_fts5.sql
-- Create an external-content FTS5 virtual table on card_faces to avoid duplicating text data.

CREATE VIRTUAL TABLE card_faces_fts USING fts5(
    name,
    oracle_text,
    content='card_faces',
    content_rowid='rowid'
);

-- Triggers to automatically keep the FTS index in sync with card_faces

CREATE TRIGGER card_faces_ai AFTER INSERT ON card_faces
BEGIN
    INSERT INTO card_faces_fts(rowid, name, oracle_text)
    VALUES (new.rowid, new.name, new.oracle_text);
END;

CREATE TRIGGER card_faces_ad AFTER DELETE ON card_faces
BEGIN
    INSERT INTO card_faces_fts(card_faces_fts, rowid, name, oracle_text)
    VALUES ('delete', old.rowid, old.name, old.oracle_text);
END;

CREATE TRIGGER card_faces_au AFTER UPDATE ON card_faces
BEGIN
    INSERT INTO card_faces_fts(card_faces_fts, rowid, name, oracle_text)
    VALUES ('delete', old.rowid, old.name, old.oracle_text);
    INSERT INTO card_faces_fts(rowid, name, oracle_text)
    VALUES (new.rowid, new.name, new.oracle_text);
END;
