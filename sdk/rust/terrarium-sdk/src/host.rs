//! Raw host imports — must match `docs/bytecode.md` / sim ABI.

#[link(wasm_import_module = "terrarium")]
extern "C" {
    pub fn sleep();
    pub fn energy() -> i64;
    pub fn health() -> i64;
    pub fn pos_x() -> i32;
    pub fn pos_y() -> i32;
    pub fn facing() -> i32;
    pub fn rotate(delta: i32) -> i32;
    pub fn sense(dq: i32, dr: i32, ptr: i32) -> i32;
    #[link_name = "move"]
    pub fn step(_forward: i32) -> i32;
    pub fn dig(rel: i32) -> i32;
    pub fn place(rel: i32) -> i32;
    pub fn eat(rel: i32) -> i32;
    pub fn hit(rel: i32) -> i32;
    pub fn spawn(rel: i32, energy: i32) -> i32;
    pub fn suicide();
    pub fn signal_broadcast(byte: i32) -> i32;
    pub fn signal_to(_ptr: i32, _byte: i32) -> i32;
    pub fn recv(_ptr: i32) -> i32;
    pub fn random_byte() -> i32;
    pub fn uptime() -> i32;
}
