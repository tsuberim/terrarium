CREATE TABLE IF NOT EXISTS creatures (
    id TEXT PRIMARY KEY,
    owner_uid TEXT NOT NULL REFERENCES accounts (firebase_uid),
    x INTEGER NOT NULL,
    y INTEGER NOT NULL,
    energy INTEGER NOT NULL CHECK (energy >= 0),
    code TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (x, y)
);

CREATE INDEX IF NOT EXISTS idx_creatures_owner ON creatures (owner_uid);
