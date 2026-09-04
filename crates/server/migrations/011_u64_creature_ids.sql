-- u64 creature ids (same namespace as account_creature_id) + owner_id column

ALTER TABLE accounts ADD COLUMN account_creature_id INTEGER;

CREATE UNIQUE INDEX IF NOT EXISTS idx_accounts_creature_id ON accounts (account_creature_id)
WHERE
    account_creature_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS id_sequence (
    name TEXT PRIMARY KEY,
    next_val INTEGER NOT NULL
);

INSERT INTO id_sequence (name, next_val) VALUES ('global_id', 10000);

CREATE TABLE creatures_new (
    id INTEGER PRIMARY KEY,
    owner_uid TEXT NOT NULL REFERENCES accounts (firebase_uid),
    owner_id INTEGER NOT NULL DEFAULT 0,
    x INTEGER NOT NULL,
    y INTEGER NOT NULL,
    energy INTEGER NOT NULL CHECK (energy >= 0),
    health INTEGER NOT NULL DEFAULT 100,
    max_health INTEGER NOT NULL DEFAULT 100,
    code TEXT NOT NULL,
    bytecode BLOB,
    born_tick INTEGER NOT NULL DEFAULT 0,
    facing INTEGER NOT NULL DEFAULT 0,
    pc INTEGER NOT NULL DEFAULT 0,
    stack BLOB NOT NULL DEFAULT x'',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (x, y)
);

INSERT INTO creatures_new (
    id,
    owner_uid,
    owner_id,
    x,
    y,
    energy,
    health,
    max_health,
    code,
    bytecode,
    born_tick,
    facing,
    pc,
    stack,
    created_at
)
SELECT
    10000 + ROW_NUMBER() OVER (ORDER BY rowid) - 1,
    owner_uid,
    10000 + ROW_NUMBER() OVER (ORDER BY rowid) - 1,
    x,
    y,
    energy,
    health,
    max_health,
    code,
    bytecode,
    born_tick,
    facing,
    pc,
    stack,
    created_at
FROM creatures;

UPDATE id_sequence
SET
    next_val = (
        SELECT COALESCE(MAX(id), 9999) + 1
        FROM creatures_new
    )
WHERE
    name = 'global_id';

DROP TABLE creatures;

ALTER TABLE creatures_new RENAME TO creatures;

CREATE INDEX IF NOT EXISTS idx_creatures_owner ON creatures (owner_uid);
