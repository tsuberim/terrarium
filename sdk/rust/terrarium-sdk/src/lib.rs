//! Terrarium creature SDK (ABI v2).

#![no_std]

pub mod host;
pub mod mem;

pub use mem::{Payload, Rel, TileView, ABI_ACTION, ABI_INIT, ABI_RECV, PAYLOAD_SIZE, RECV_MSG_SIZE};

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

pub fn rand() -> u64 {
    unsafe { host::rand() }
}

pub fn energy() -> i64 {
    mem::state_i64(mem::off::ENERGY)
}

pub fn health() -> i32 {
    mem::state_i32(mem::off::HEALTH)
}

pub fn pos_x() -> i32 {
    mem::state_i32(mem::off::POS_X)
}

pub fn pos_y() -> i32 {
    mem::state_i32(mem::off::POS_Y)
}

pub fn facing() -> u32 {
    mem::state_u32(mem::off::FACING)
}

pub fn uptime() -> u32 {
    mem::state_u32(mem::off::UPTIME)
}

pub fn owner_id() -> u64 {
    mem::state_u64(mem::off::OWNER_ID)
}

pub fn id() -> u64 {
    mem::state_u64(mem::off::ID)
}

pub fn inbox_len() -> u32 {
    mem::state_u32(mem::off::INBOX_LEN)
}

pub fn init_payload() -> Payload {
    mem::read_init()
}

pub fn tile(rel: Rel) -> TileView {
    mem::rel_tile(rel)
}

pub fn move_rel(rel: Rel) -> i32 {
    let mut p = Payload::new(action::MOVE);
    p.set_rel(rel as u8);
    host::submit(&p)
}

pub fn move_forward() -> i32 {
    move_rel(Rel::Fwd)
}

pub fn eat_rel(rel: Rel) -> i32 {
    let mut p = Payload::new(action::EAT);
    p.set_rel(rel as u8);
    host::submit(&p)
}

pub fn eat_forward() -> i32 {
    eat_rel(Rel::Fwd)
}

pub fn rotate(delta: i32) -> i32 {
    let mut p = Payload::new(action::ROTATE);
    p.set_a(delta as u64);
    host::submit(&p)
}

pub fn dig(rel: Rel) -> i32 {
    let mut p = Payload::new(action::DIG);
    p.set_rel(rel as u8);
    host::submit(&p)
}

pub fn place(rel: Rel) -> i32 {
    let mut p = Payload::new(action::PLACE);
    p.set_rel(rel as u8);
    host::submit(&p)
}

pub fn hit(rel: Rel) -> i32 {
    let mut p = Payload::new(action::HIT);
    p.set_rel(rel as u8);
    host::submit(&p)
}

pub fn spawn(rel: Rel, energy: i64, owner: u64, child: &[u8; 40]) -> i32 {
    let mut p = Payload::new(action::SPAWN);
    p.set_rel(rel as u8);
    p.set_a(energy as u64);
    p.set_spawn_data(owner, child);
    host::submit(&p)
}

pub fn signal(target: u64, msg: &Payload) -> i32 {
    let mut p = *msg;
    p.set_tag(action::SIGNAL);
    p.set_a(target);
    host::submit(&p)
}

pub fn broadcast(msg: &Payload) -> i32 {
    let mut p = *msg;
    p.set_tag(action::BROADCAST);
    host::submit(&p)
}

pub fn recv(msg: &mut RecvMsg) -> bool {
    if unsafe { host::recv() } == 0 {
        return false;
    }
    *msg = mem::read_recv();
    true
}

#[repr(C)]
pub struct RecvMsg {
    pub sender: u64,
    pub payload: Payload,
}

pub mod prelude {
    pub use crate::{
        action, broadcast, eat_forward, eat_rel, energy, facing, health, hit, id, inbox_len,
        init_payload, move_forward, move_rel, owner_id, place, pos_x, pos_y, rand, recv, rotate, signal,
        spawn, tile, uptime, Payload, RecvMsg, Rel, TileView,
    };
}
