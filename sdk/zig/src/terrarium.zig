//! Terrarium host syscalls — must match docs/bytecode.md import names/signatures.

pub const TileKind = enum(i32) {
    empty = 0,
    solid = 1,
    creature = 2,
    corpse = 3,
    food = 4,
};

extern "terrarium" fn sleep() void;
extern "terrarium" fn energy() i64;
extern "terrarium" fn health() i64;
extern "terrarium" fn pos_x() i32;
extern "terrarium" fn pos_y() i32;
extern "terrarium" fn facing() i32;
extern "terrarium" fn rotate(delta: i32) i32;
extern "terrarium" fn sense(dq: i32, dr: i32, ptr: i32) i32;
extern "terrarium" fn move(rel: i32) i32;
extern "terrarium" fn dig(rel: i32) i32;
extern "terrarium" fn place(rel: i32) i32;
extern "terrarium" fn eat(rel: i32) i32;
extern "terrarium" fn hit(rel: i32) i32;
extern "terrarium" fn spawn(rel: i32, energy: i32) i32;
extern "terrarium" fn suicide() void;
extern "terrarium" fn signal_broadcast(byte: i32) i32;
extern "terrarium" fn signal_to(ptr: i32, byte: i32) i32;
extern "terrarium" fn recv(ptr: i32) i32;
extern "terrarium" fn random_byte() i32;
extern "terrarium" fn uptime() i32;

/// 24-byte sense struct (kind @0, orientation @4, energy @8, health @16, max_health @20).
var sense_buf: [24]u8 align(4) = undefined;

fn read_i32_at(offset: usize) i32 {
    const bytes: *align(1) const [4]u8 = sense_buf[offset..][0..4];
    return @bitCast(bytes.*);
}

/// Sense a cell at axial offset (dq, dr). Returns tile kind or null if out of FOV.
pub fn sense_at(dq: i32, dr: i32) ?TileKind {
    const ptr: i32 = @intFromPtr(&sense_buf);
    if (sense(dq, dr, ptr) == 0) return null;
    return @enumFromInt(read_i32_at(0));
}

pub fn sleep_host() void {
    sleep();
}

pub fn move_forward() i32 {
    return move(0);
}

pub fn eat_forward() i32 {
    return eat(0);
}

pub fn energy_host() i64 {
    return energy();
}

pub fn spawn_forward(gift: i32) i32 {
    return spawn(0, gift);
}

pub fn signal_host(byte: u8) i32 {
    return signal_broadcast(byte);
}

pub fn random_host() u8 {
    return @truncate(random_byte());
}
