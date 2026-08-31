//! WebSocket wire types and delta builders.

use std::collections::{HashMap, HashSet};

use serde::Serialize;
use terrarium_kernel::{
    vm::Creature, CreatureAction, DeathReason, EnergyLedger, SimConfig, WorldEvent, WorldTile,
    WorldTiles,
};

#[derive(Clone, Serialize)]
pub struct CreaturePublic {
    pub id: String,
    pub x: i64,
    pub y: i64,
    pub energy: i64,
    pub health: i32,
    pub max_health: i32,
    pub owner_uid: String,
    pub facing: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub program_hash: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct TilePublic {
    pub x: i64,
    pub y: i64,
    pub kind: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub energy: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub death_reason: Option<DeathReason>,
}

#[derive(Clone, Serialize)]
pub struct EnergyLedgerPublic {
    pub destroyed: i64,
    pub free_minted: i64,
    pub free_budget: i64,
}

#[derive(Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorldMessage {
    Delta {
        tick: u64,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        full: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        deploy_cost: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        corpse_energy: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        sim_config: Option<SimConfig>,
        #[serde(skip_serializing_if = "Option::is_none")]
        energy_ledger: Option<EnergyLedgerPublic>,
        creatures_upsert: Vec<CreaturePublic>,
        creatures_remove: Vec<String>,
        tiles_upsert: Vec<TilePublic>,
        tiles_remove: Vec<[i64; 2]>,
        actions: Vec<CreatureAction>,
        events: Vec<WorldEvent>,
    },
}

impl WorldMessage {
    pub fn tick_delta(
        tick: u64,
        creatures_upsert: Vec<CreaturePublic>,
        creatures_remove: Vec<String>,
        tiles_upsert: Vec<TilePublic>,
        tiles_remove: Vec<[i64; 2]>,
        actions: Vec<CreatureAction>,
        events: Vec<WorldEvent>,
    ) -> Self {
        Self::Delta {
            tick,
            full: false,
            deploy_cost: None,
            corpse_energy: None,
            sim_config: None,
            energy_ledger: None,
            creatures_upsert,
            creatures_remove,
            tiles_upsert,
            tiles_remove,
            actions,
            events,
        }
    }
}

pub fn program_hash(wasm: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(wasm);
    digest[..8].iter().map(|b| format!("{b:02x}")).collect()
}

pub fn creature_public(c: &Creature) -> CreaturePublic {
    CreaturePublic {
        id: c.id.clone(),
        x: c.x as i64,
        y: c.y as i64,
        energy: c.energy,
        health: c.health,
        max_health: c.max_health,
        owner_uid: c.owner_uid.clone(),
        facing: c.facing,
        program_hash: Some(program_hash(&c.wasm)),
    }
}

pub fn tile_public(pos: (i32, i32), tile: WorldTile) -> TilePublic {
    match tile {
        WorldTile::Solid => TilePublic {
            x: pos.0 as i64,
            y: pos.1 as i64,
            kind: 1,
            energy: None,
            death_reason: None,
        },
        WorldTile::Corpse {
            energy,
            death_reason,
        } => TilePublic {
            x: pos.0 as i64,
            y: pos.1 as i64,
            kind: 3,
            energy: Some(energy),
            death_reason: Some(death_reason),
        },
        WorldTile::Food { energy } => TilePublic {
            x: pos.0 as i64,
            y: pos.1 as i64,
            kind: 4,
            energy: Some(energy),
            death_reason: None,
        },
    }
}

pub fn ledger_public(ledger: &EnergyLedger) -> EnergyLedgerPublic {
    EnergyLedgerPublic {
        destroyed: ledger.destroyed,
        free_minted: ledger.free_minted,
        free_budget: ledger.free_budget(),
    }
}

pub fn tiles_public(tiles: &WorldTiles) -> Vec<TilePublic> {
    let mut out: Vec<_> = tiles
        .iter()
        .map(|(pos, tile)| tile_public(*pos, *tile))
        .collect();
    out.sort_by_key(|t| (t.x, t.y));
    out
}

pub fn build_creature_delta(
    before: &HashMap<String, (i32, i32, i64, i32, i32, u8)>,
    after: &[Creature],
) -> (Vec<CreaturePublic>, Vec<String>) {
    let after_ids: HashSet<&str> = after.iter().map(|c| c.id.as_str()).collect();

    let mut creatures_remove = Vec::new();
    for id in before.keys() {
        if !after_ids.contains(id.as_str()) {
            creatures_remove.push(id.clone());
        }
    }

    let mut creatures_upsert = Vec::new();
    for c in after {
        let changed = before
            .get(&c.id)
            .map(|(x, y, e, h, mh, f)| {
                *x != c.x
                    || *y != c.y
                    || *e != c.energy
                    || *h != c.health
                    || *mh != c.max_health
                    || *f != c.facing
            })
            .unwrap_or(true);
        if changed {
            creatures_upsert.push(creature_public(c));
        }
    }

    (creatures_upsert, creatures_remove)
}

pub fn build_tile_delta(
    tiles_dirty: &HashSet<(i32, i32)>,
    tiles: &WorldTiles,
) -> (Vec<TilePublic>, Vec<[i64; 2]>) {
    let mut tiles_upsert = Vec::new();
    let mut tiles_remove = Vec::new();
    for &(x, y) in tiles_dirty {
        if let Some(tile) = tiles.get(&(x, y)) {
            tiles_upsert.push(tile_public((x, y), *tile));
        } else {
            tiles_remove.push([x as i64, y as i64]);
        }
    }
    tiles_upsert.sort_by_key(|t| (t.x, t.y));
    tiles_remove.sort_by(|[ax, ay], [bx, by]| ax.cmp(bx).then(ay.cmp(by)));
    (tiles_upsert, tiles_remove)
}
