//! Host-side writes of ABI v2 guest memory regions.

use wasmtime::{Memory, Store};

use crate::abi::{
    self, write_i32, write_i64, write_u32, write_u64, Payload, ABI_BASE, ABI_INIT, ABI_VERSION,
    MEM_MAGIC, PAYLOAD_SIZE, REL_TILES_OFFSET, REL_TILE_COUNT, STATE_SIZE, TILE_VIEW_SIZE,
    VISION_ENTRY_SIZE, VISION_MAX, VISION_OFFSET,
};
use crate::events::DeathReason;
use crate::hex;
use crate::host::HostState;
use crate::vm::Snapshot;
use crate::world_tile::{sense_kind, WorldTile, WorldTiles};

pub fn write_creature_init(store: &mut Store<HostState>, init: &Payload) {
    if let Some(memory) = store.data().memory {
        write_bytes_store(store, memory, ABI_INIT, &init.bytes);
    }
}

pub fn refresh_creature_memory(store: &mut Store<HostState>) {
    let memory = match store.data().memory {
        Some(m) => m,
        None => return,
    };
    let creature = unsafe { &*store.data().creature };
    let snapshot = unsafe { &*store.data().snapshot };
    let tiles = unsafe { &*store.data().tiles };
    let config = unsafe { &*store.data().config };
    let tick = store.data().tick;
    let facing = store.data().result.effective_facing(creature.facing);
    let inbox_len = creature.inbox.len() as u32;

    let mut state = [0u8; STATE_SIZE];
    write_u32(&mut state, abi::state_off::MAGIC as usize, MEM_MAGIC);
    write_u32(&mut state, abi::state_off::VERSION as usize, ABI_VERSION);
    write_u64(&mut state, abi::state_off::ID as usize, creature.id);
    write_u64(
        &mut state,
        abi::state_off::OWNER_ID as usize,
        creature.owner_id,
    );
    write_u64(&mut state, abi::state_off::TICK as usize, tick);
    write_i32(&mut state, abi::state_off::POS_X as usize, creature.x);
    write_i32(&mut state, abi::state_off::POS_Y as usize, creature.y);
    write_u32(
        &mut state,
        abi::state_off::FACING as usize,
        u32::from(facing),
    );
    write_i64(&mut state, abi::state_off::ENERGY as usize, creature.energy);
    write_i32(&mut state, abi::state_off::HEALTH as usize, creature.health);
    write_i32(
        &mut state,
        abi::state_off::MAX_HEALTH as usize,
        creature.max_health,
    );
    write_u32(
        &mut state,
        abi::state_off::UPTIME as usize,
        tick.saturating_sub(creature.born_tick) as u32,
    );
    write_u32(&mut state, abi::state_off::INBOX_LEN as usize, inbox_len);

    write_bytes_store(store, memory, ABI_BASE, &state);

    for rel in 0..REL_TILE_COUNT as u8 {
        let abs = hex::abs_dir(facing, rel);
        let (dq, dr) = hex::neighbor(0, 0, abs).unwrap_or((0, 0));
        let tile = tile_view(snapshot, tiles, creature.x + dq, creature.y + dr);
        let off = ABI_BASE + REL_TILES_OFFSET + u32::from(rel) * TILE_VIEW_SIZE as u32;
        write_bytes_store(store, memory, off, &tile);
    }

    let mut vision_buf = vec![0u8; VISION_MAX * VISION_ENTRY_SIZE];
    let mut vision_count = 0usize;
    let r = config.r_vis;
    for dq in -r..=r {
        for dr in -r..=r {
            if !config.in_hex_range(dq, dr, r) {
                continue;
            }
            if hex::distance(0, 0, dq, dr) <= 1 {
                continue;
            }
            if !hex::in_fov(facing, dq, dr, config.vis_half_arc) {
                continue;
            }
            if vision_count >= VISION_MAX {
                break;
            }
            let x = creature.x + dq;
            let y = creature.y + dr;
            let view = tile_view(snapshot, tiles, x, y);
            let entry_off = vision_count * VISION_ENTRY_SIZE;
            write_i32(&mut vision_buf, entry_off, dq);
            write_i32(&mut vision_buf, entry_off + 4, dr);
            vision_buf[entry_off + 8..entry_off + 8 + TILE_VIEW_SIZE].copy_from_slice(&view);
            vision_count += 1;
        }
    }
    write_u32(
        &mut state,
        abi::state_off::VISION_COUNT as usize,
        vision_count as u32,
    );
    write_bytes_store(store, memory, ABI_BASE, &state);
    if vision_count > 0 {
        write_bytes_store(
            store,
            memory,
            VISION_OFFSET,
            &vision_buf[..vision_count * VISION_ENTRY_SIZE],
        );
    }

    let zero = [0u8; PAYLOAD_SIZE];
    write_bytes_store(store, memory, abi::ABI_ACTION, &zero);
}

fn death_reason_code(reason: DeathReason) -> u64 {
    match reason {
        DeathReason::EnergyFloor => 0,
        DeathReason::OutOfEnergy => 1,
        DeathReason::OutOfGas => 2,
        DeathReason::EmptyProgram => 3,
        DeathReason::InvalidProgram => 4,
        DeathReason::WasmTrap => 5,
        DeathReason::OutOfVision => 6,
        DeathReason::BadDirection => 7,
        DeathReason::SpawnEnergyTooLow => 8,
        DeathReason::SignalUnknownTarget => 9,
        DeathReason::SignalOutOfRange => 10,
        DeathReason::Suicide => 11,
        DeathReason::SpawnFailed => 12,
        DeathReason::SignalFailed => 13,
        DeathReason::Killed => 14,
        DeathReason::Eaten => 15,
    }
}

fn tile_view(snapshot: &Snapshot, tiles: &WorldTiles, x: i32, y: i32) -> [u8; TILE_VIEW_SIZE] {
    let mut out = [0u8; TILE_VIEW_SIZE];
    let has_creature = snapshot.id_at.contains_key(&(x, y));
    let kind = sense_kind(tiles, x, y, has_creature);
    write_u64(&mut out, tile_off_usize(abi::tile_off::KIND), kind);
    if let Some(id) = snapshot.id_at.get(&(x, y)) {
        write_u64(&mut out, tile_off_usize(abi::tile_off::ENTITY_ID), *id);
        write_i64(
            &mut out,
            tile_off_usize(abi::tile_off::ENERGY),
            snapshot.energy.get(id).copied().unwrap_or(0),
        );
        write_i32(
            &mut out,
            tile_off_usize(abi::tile_off::HEALTH),
            snapshot.health.get(id).copied().unwrap_or(0),
        );
        write_i32(
            &mut out,
            tile_off_usize(abi::tile_off::MAX_HEALTH),
            snapshot.max_health.get(id).copied().unwrap_or(0),
        );
        write_u32(
            &mut out,
            tile_off_usize(abi::tile_off::FACING),
            u32::from(snapshot.facing.get(id).copied().unwrap_or(0)),
        );
    } else if let Some(tile) = tiles.get(&(x, y)) {
        match tile {
            WorldTile::Corpse {
                energy,
                death_reason,
            } => {
                write_i64(&mut out, tile_off_usize(abi::tile_off::ENERGY), *energy);
                write_u64(
                    &mut out,
                    tile_off_usize(abi::tile_off::AUX),
                    death_reason_code(*death_reason),
                );
            }
            WorldTile::Food { energy } => {
                write_i64(&mut out, tile_off_usize(abi::tile_off::ENERGY), *energy);
            }
            WorldTile::Solid => {}
        }
    }
    out
}

fn tile_off_usize(off: u32) -> usize {
    off as usize
}

fn write_bytes_store(store: &mut Store<HostState>, memory: Memory, off: u32, bytes: &[u8]) {
    let base = off as usize;
    let end = base + bytes.len();
    let data = memory.data_mut(store);
    if end <= data.len() {
        data[base..end].copy_from_slice(bytes);
    }
}
