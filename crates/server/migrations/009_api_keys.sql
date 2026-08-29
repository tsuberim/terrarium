CREATE TABLE IF NOT EXISTS api_keys (
    id TEXT PRIMARY KEY,
    owner_uid TEXT NOT NULL REFERENCES accounts (firebase_uid) ON DELETE CASCADE,
    name TEXT NOT NULL DEFAULT '',
    prefix TEXT NOT NULL,
    key_hash TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_used_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_api_keys_owner ON api_keys (owner_uid);
