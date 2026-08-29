ALTER TABLE world_tiles ADD COLUMN energy INTEGER CHECK (energy IS NULL OR energy >= 0);
