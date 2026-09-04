//! Guest memory reads for ABI v2.

pub const ABI_BASE: u32 = 4096;
pub const ABI_INIT: u32 = 8192;
pub const ABI_RECV: u32 = 8256;
pub const ABI_ACTION: u32 = 8328;
pub const PAYLOAD_SIZE: usize = 64;
pub const PAYLOAD_DATA: usize = 48;
pub const RECV_MSG_SIZE: usize = 72;
pub const TILE_VIEW_SIZE: usize = 48;

pub mod off {
    pub const ID: u32 = 8;
    pub const OWNER_ID: u32 = 16;
    pub const POS_X: u32 = 32;
    pub const POS_Y: u32 = 36;
    pub const FACING: u32 = 40;
    pub const ENERGY: u32 = 44;
    pub const HEALTH: u32 = 52;
    pub const MAX_HEALTH: u32 = 56;
    pub const UPTIME: u32 = 60;
    pub const INBOX_LEN: u32 = 64;
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rel {
    Fwd = 0,
    FwdR = 1,
    BackR = 2,
    Back = 3,
    BackL = 4,
    FwdL = 5,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TileView {
    bytes: [u8; TILE_VIEW_SIZE],
}

impl TileView {
    pub fn kind(&self) -> u64 {
        read_u64(&self.bytes, 0)
    }

    pub fn energy(&self) -> i64 {
        read_i64(&self.bytes, 8)
    }

    pub fn entity_id(&self) -> u64 {
        read_u64(&self.bytes, 32)
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Payload {
    pub(crate) bytes: [u8; PAYLOAD_SIZE],
}

impl Default for Payload {
    fn default() -> Self {
        Self { bytes: [0; PAYLOAD_SIZE] }
    }
}

impl Payload {
    pub fn new(tag: u32) -> Self {
        let mut p = Self::default();
        p.set_tag(tag);
        p
    }

    pub fn tag(&self) -> u32 {
        read_u32(&self.bytes, 0)
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

    pub fn data(&self) -> &[u8; PAYLOAD_DATA] {
        unsafe { &*self.bytes[16..].as_ptr().cast() }
    }

    pub fn set_data(&mut self, data: &[u8; PAYLOAD_DATA]) {
        self.bytes[16..].copy_from_slice(data);
    }

    pub fn set_spawn_data(&mut self, owner_id: u64, child: &[u8; 40]) {
        write_u64(&mut self.bytes, 16, owner_id);
        self.bytes[24..64].copy_from_slice(child);
    }
}

pub fn write_action(payload: &Payload) {
    unsafe {
        core::ptr::copy_nonoverlapping(
            payload.bytes.as_ptr(),
            ABI_ACTION as *mut u8,
            PAYLOAD_SIZE,
        );
    }
}

pub fn state_i32(off: u32) -> i32 {
    read_i32_at(ABI_BASE + off)
}

pub fn state_u32(off: u32) -> u32 {
    read_u32_at(ABI_BASE + off)
}

pub fn state_i64(off: u32) -> i64 {
    read_i64_at(ABI_BASE + off)
}

pub fn state_u64(off: u32) -> u64 {
    read_u64_at(ABI_BASE + off)
}

pub fn read_init() -> Payload {
    let mut bytes = [0u8; PAYLOAD_SIZE];
    unsafe {
        core::ptr::copy_nonoverlapping(ABI_INIT as *const u8, bytes.as_mut_ptr(), PAYLOAD_SIZE);
    }
    Payload { bytes }
}

pub fn read_recv() -> crate::RecvMsg {
    let mut msg = crate::RecvMsg {
        sender: read_u64_at(ABI_RECV),
        payload: Payload { bytes: [0; PAYLOAD_SIZE] },
    };
    unsafe {
        core::ptr::copy_nonoverlapping(
            (ABI_RECV + 8) as *const u8,
            msg.payload.bytes.as_mut_ptr(),
            PAYLOAD_SIZE,
        );
    }
    msg
}

pub fn rel_tile(rel: Rel) -> TileView {
    let base = ABI_BASE + 72 + (rel as u32) * TILE_VIEW_SIZE as u32;
    TileView {
        bytes: read_block(base),
    }
}

fn read_block(base: u32) -> [u8; TILE_VIEW_SIZE] {
    let mut out = [0u8; TILE_VIEW_SIZE];
    let ptr = base as usize;
    unsafe {
        core::ptr::copy_nonoverlapping(ptr as *const u8, out.as_mut_ptr(), TILE_VIEW_SIZE);
    }
    out
}

fn read_i32_at(base: u32) -> i32 {
    let mut b = [0u8; 4];
    unsafe {
        core::ptr::copy_nonoverlapping(base as *const u8, b.as_mut_ptr(), 4);
    }
    i32::from_le_bytes(b)
}

fn read_u32_at(base: u32) -> u32 {
    u32::from_le_bytes(read4(base))
}

fn read_i64_at(base: u32) -> i64 {
    let mut b = [0u8; 8];
    unsafe {
        core::ptr::copy_nonoverlapping(base as *const u8, b.as_mut_ptr(), 8);
    }
    i64::from_le_bytes(b)
}

fn read_u64_at(base: u32) -> u64 {
    u64::from_le_bytes(read8(base))
}

fn read4(base: u32) -> [u8; 4] {
    let mut b = [0u8; 4];
    unsafe {
        core::ptr::copy_nonoverlapping(base as *const u8, b.as_mut_ptr(), 4);
    }
    b
}

fn read8(base: u32) -> [u8; 8] {
    let mut b = [0u8; 8];
    unsafe {
        core::ptr::copy_nonoverlapping(base as *const u8, b.as_mut_ptr(), 8);
    }
    b
}

fn read_u64(buf: &[u8], off: usize) -> u64 {
    u64::from_le_bytes(buf[off..off + 8].try_into().unwrap())
}

fn read_i64(buf: &[u8], off: usize) -> i64 {
    i64::from_le_bytes(buf[off..off + 8].try_into().unwrap())
}

fn read_u32(buf: &[u8], off: usize) -> u32 {
    u32::from_le_bytes(buf[off..off + 4].try_into().unwrap())
}

fn write_u32(buf: &mut [u8], off: usize, v: u32) {
    buf[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

fn write_u64(buf: &mut [u8], off: usize, v: u64) {
    buf[off..off + 8].copy_from_slice(&v.to_le_bytes());
}
