CREATE TABLE IF NOT EXISTS accounts (
    firebase_uid TEXT PRIMARY KEY,
    credits INTEGER NOT NULL DEFAULT 0 CHECK (credits >= 0),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
