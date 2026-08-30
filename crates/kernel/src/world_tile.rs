use std::collections::HashMap;

use crate::abi::tile;
use crate::events::DeathReason;

pub type WorldTiles = HashMap<(i32, i32), WorldTile>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldTile {
    Solid,
    Corpse {
        energy: i64,
        death_reason: DeathReason,
    },
    Food {
        energy: i64,
    },
}

impl WorldTile {
    pub fn sense_kind(self) -> i32 {
        match self {
            WorldTile::Solid => tile::SOLID,
            WorldTile::Corpse { .. } => tile::CORPSE,
            WorldTile::Food { .. } => tile::FOOD,
        }
    }
}

pub fn count_food(tiles: &WorldTiles) -> u32 {
    tiles
        .values()
        .filter(|t| matches!(t, WorldTile::Food { .. }))
        .count() as u32
}

pub fn place_food(tiles: &mut WorldTiles, x: i32, y: i32, energy: i64) {
    if energy <= 0 {
        return;
    }
    match tiles.get(&(x, y)).copied() {
        Some(WorldTile::Solid) | Some(WorldTile::Corpse { .. }) => {}
        Some(WorldTile::Food { energy: existing }) => {
            tiles.insert((x, y), WorldTile::Food { energy: existing + energy });
        }
        None => {
            tiles.insert((x, y), WorldTile::Food { energy });
        }
    }
}

pub fn place_corpse(tiles: &mut WorldTiles, x: i32, y: i32, energy: i64, death_reason: DeathReason) {
    if energy < 0 {
        return;
    }
    match tiles.get(&(x, y)).copied() {
        Some(WorldTile::Solid) => {}
        Some(WorldTile::Corpse { energy: existing, .. }) => {
            tiles.insert(
                (x, y),
                WorldTile::Corpse {
                    energy: existing + energy,
                    death_reason,
                },
            );
        }
        None | Some(WorldTile::Food { .. }) => {
            tiles.insert((x, y), WorldTile::Corpse { energy, death_reason });
        }
    }
}

pub fn set_cell(tiles: &mut WorldTiles, x: i32, y: i32, kind: i32) {
    if kind == tile::EMPTY {
        tiles.remove(&(x, y));
    } else if kind == tile::SOLID {
        tiles.insert((x, y), WorldTile::Solid);
    }
}

pub fn sense_kind(tiles: &WorldTiles, x: i32, y: i32, creature: bool) -> i32 {
    if creature {
        return tile::CREATURE;
    }
    tiles
        .get(&(x, y))
        .map(|t| t.sense_kind())
        .unwrap_or(tile::EMPTY)
}

pub fn blocks_movement(tiles: &WorldTiles, x: i32, y: i32) -> bool {
    tiles.contains_key(&(x, y))
}
