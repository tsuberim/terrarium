//! Host ABI v2 — fixed payloads, memory-mapped state, minimal syscalls.

pub const ABI_VERSION: u32 = 2;
pub const MEM_MAGIC: u32 = 0x5452_0002;

/// Guest memory regions (bytes).
pub const ABI_BASE: u32 = 4096;
pub const STATE_SIZE: usize = 72;
pub const REL_TILES_OFFSET: u32 = STATE_SIZE as u32;
pub const REL_TILE_COUNT: usize = 6;
pub const TILE_VIEW_SIZE: usize = 48;
pub const REL_TILES_SIZE: usize = REL_TILE_COUNT * TILE_VIEW_SIZE;
pub const VISION_OFFSET: u32 = REL_TILES_OFFSET + REL_TILES_SIZE as u32;
pub const VISION_ENTRY_SIZE: usize = 56;
pub const VISION_MAX: usize = 91;
pub const VISION_MAX_BYTES: usize = VISION_MAX * VISION_ENTRY_SIZE;

/// Birth init + action/signal/spawn messages.
pub const PAYLOAD_SIZE: usize = 64;
pub const PAYLOAD_HEADER: usize = 16;
pub const PAYLOAD_DATA: usize = 48;
/// `recv` writes sender u64 + payload.
pub const RECV_MSG_SIZE: usize = 8 + PAYLOAD_SIZE;

pub const ABI_INIT: u32 = 8192;
/// Host-written inbox pop (`recv`); guest reads after return 1.
pub const ABI_RECV: u32 = ABI_INIT + PAYLOAD_SIZE as u32;
/// Guest-written pending action (`act()` reads here).
pub const ABI_ACTION: u32 = ABI_RECV + RECV_MSG_SIZE as u32;

/// Absolute world directions (pointy-top axial): E, NE, NW, W, SW, SE.
pub mod dir {
    pub const E: i32 = 0;
    pub const NE: i32 = 1;
    pub const NW: i32 = 2;
    pub const W: i32 = 3;
    pub const SW: i32 = 4;
    pub const SE: i32 = 5;
    pub const COUNT: i32 = 6;
}

pub mod rel {
    pub const FWD: u8 = 0;
    pub const FWD_R: u8 = 1;
    pub const BACK_R: u8 = 2;
    pub const BACK: u8 = 3;
    pub const BACK_L: u8 = 4;
    pub const FWD_L: u8 = 5;
    pub const COUNT: u8 = 6;

    pub fn valid(r: u8) -> bool {
        r < COUNT
    }
}

pub mod action {
    pub const NONE: u32 = 0;
    pub const MOVE: u32 = 1;
    pub const ROTATE: u32 = 2;
    pub const DIG: u32 = 3;
    pub const PLACE: u32 = 4;
    pub const EAT: u32 = 5;
    pub const HIT: u32 = 6;
    pub const SPAWN: u32 = 7;
    pub const SIGNAL: u32 = 8;
    pub const BROADCAST: u32 = 9;
}

pub mod tile {
    pub const EMPTY: u64 = 0;
    pub const SOLID: u64 = 1;
    pub const CREATURE: u64 = 2;
    pub const CORPSE: u64 = 3;
    pub const FOOD: u64 = 4;
}

pub mod state_off {
    pub const MAGIC: u32 = 0;
    pub const VERSION: u32 = 4;
    pub const ID: u32 = 8;
    pub const OWNER_ID: u32 = 16;
    pub const TICK: u32 = 24;
    pub const POS_X: u32 = 32;
    pub const POS_Y: u32 = 36;
    pub const FACING: u32 = 40;
    pub const ENERGY: u32 = 44;
    pub const HEALTH: u32 = 52;
    pub const MAX_HEALTH: u32 = 56;
    pub const UPTIME: u32 = 60;
    pub const INBOX_LEN: u32 = 64;
    pub const VISION_COUNT: u32 = 68;
}

pub mod tile_off {
    pub const KIND: u32 = 0;
    pub const ENERGY: u32 = 8;
    pub const HEALTH: u32 = 16;
    pub const MAX_HEALTH: u32 = 20;
    pub const FACING: u32 = 24;
    pub const ENTITY_ID: u32 = 32;
    pub const AUX: u32 = 40;
}

/// Fixed 64-byte action / init / message payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Payload {
    pub bytes: [u8; PAYLOAD_SIZE],
}

impl Default for Payload {
    fn default() -> Self {
        Self {
            bytes: [0; PAYLOAD_SIZE],
        }
    }
}

impl Payload {
    pub fn tag(&self) -> u32 {
        read_u32(&self.bytes, 0)
    }

    pub fn rel(&self) -> u8 {
        self.bytes[4]
    }

    pub fn a(&self) -> u64 {
        read_u64(&self.bytes, 8)
    }

    pub fn data(&self) -> &[u8; PAYLOAD_DATA] {
        // SAFETY: fixed slice length
        unsafe { &*self.bytes[PAYLOAD_HEADER..].as_ptr().cast() }
    }

    pub fn set_tag(&mut self, tag: u32) {
        write_u32(&mut self.bytes, 0, tag);
    }

    pub fn set_rel(&mut self, rel: u8) {
        self.bytes[4] = rel;
    }

    pub fn set_a(&mut self, a: u64) {
        write_u64(&mut self.bytes, 8, a);
    }

    pub fn set_data(&mut self, data: &[u8; PAYLOAD_DATA]) {
        self.bytes[PAYLOAD_HEADER..].copy_from_slice(data);
    }

    pub fn spawn_owner_id(&self) -> u64 {
        read_u64(&self.bytes, PAYLOAD_HEADER)
    }

    pub fn child_init_from_spawn(&self) -> Payload {
        let mut init = Payload::default();
        init.bytes[PAYLOAD_HEADER..PAYLOAD_HEADER + 40]
            .copy_from_slice(&self.bytes[PAYLOAD_HEADER + 8..PAYLOAD_HEADER + 48]);
        init
    }
}

/// Reassemble a payload from WASM scalar syscall args (legacy control bridge).
#[allow(clippy::too_many_arguments)]
pub fn payload_from_scalars(
    tag: u32,
    rel: i32,
    a: u64,
    b: u64,
    c: u64,
    d: u64,
    e: u64,
    f: u64,
    g: u64,
) -> Payload {
    let mut p = Payload::default();
    p.set_tag(tag);
    p.set_rel(rel as u8);
    p.set_a(a);
    write_u64(&mut p.bytes, PAYLOAD_HEADER, b);
    write_u64(&mut p.bytes, PAYLOAD_HEADER + 8, c);
    write_u64(&mut p.bytes, PAYLOAD_HEADER + 16, d);
    write_u64(&mut p.bytes, PAYLOAD_HEADER + 24, e);
    write_u64(&mut p.bytes, PAYLOAD_HEADER + 32, f);
    write_u64(&mut p.bytes, PAYLOAD_HEADER + 40, g);
    p
}

pub fn read_u32(buf: &[u8], off: usize) -> u32 {
    u32::from_le_bytes(buf[off..off + 4].try_into().expect("u32"))
}

pub fn read_u64(buf: &[u8], off: usize) -> u64 {
    u64::from_le_bytes(buf[off..off + 8].try_into().expect("u64"))
}

pub fn read_i32(buf: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(buf[off..off + 4].try_into().expect("i32"))
}

pub fn read_i64(buf: &[u8], off: usize) -> i64 {
    i64::from_le_bytes(buf[off..off + 8].try_into().expect("i64"))
}

pub fn write_u32(buf: &mut [u8], off: usize, v: u32) {
    buf[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

pub fn write_u64(buf: &mut [u8], off: usize, v: u64) {
    buf[off..off + 8].copy_from_slice(&v.to_le_bytes());
}

pub fn write_i32(buf: &mut [u8], off: usize, v: i32) {
    buf[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

pub fn write_i64(buf: &mut [u8], off: usize, v: i64) {
    buf[off..off + 8].copy_from_slice(&v.to_le_bytes());
}

/// Base energy unit — values are in millions (corpse floor = 1M).
pub const ENERGY_SCALE: i64 = 100_000;

pub const CORPSE_ENERGY: i64 = 10 * ENERGY_SCALE;

pub const SPAWN_MIN_ENERGY: i64 = CORPSE_ENERGY + ACTION_ENERGY;

pub const CORPSE_YIELD_PERCENT: i64 = 80;

pub fn corpse_yield_energy(creature_energy: i64) -> i64 {
    creature_energy.max(0) * CORPSE_YIELD_PERCENT / 100
}

pub const OPCODES_PER_TICK: u64 = 25_000;
pub const ENERGY_PER_OPCODE: i64 = 1;
pub const ACTION_ENERGY: i64 = ENERGY_SCALE / 4;

/// New creature id — random non-zero u64.
pub fn new_creature_id() -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};
    let mut h = DefaultHasher::new();
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .hash(&mut h);
    std::thread::current().id().hash(&mut h);
    let id = h.finish();
    if id == 0 {
        1
    } else {
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_size_is_64() {
        assert_eq!(std::mem::size_of::<Payload>(), PAYLOAD_SIZE);
    }
}
