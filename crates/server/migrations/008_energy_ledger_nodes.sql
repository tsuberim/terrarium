CREATE TABLE IF NOT EXISTS energy_ledger (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    destroyed INTEGER NOT NULL DEFAULT 0,
    free_minted INTEGER NOT NULL DEFAULT 0
);
INSERT OR IGNORE INTO energy_ledger (id, destroyed, free_minted) VALUES (1, 0, 0);

CREATE TABLE world_tiles_new (
    x INTEGER NOT NULL,
    y INTEGER NOT NULL,
    kind INTEGER NOT NULL CHECK (kind IN (1, 3, 4)),
    energy INTEGER CHECK (energy IS NULL OR energy >= 0),
    death_reason TEXT,
    PRIMARY KEY (x, y)
);
INSERT INTO world_tiles_new (x, y, kind, energy, death_reason)
SELECT x, y, kind, energy, death_reason FROM world_tiles;
DROP TABLE world_tiles;
ALTER TABLE world_tiles_new RENAME TO world_tiles;
