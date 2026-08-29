//! Terrarium v0 instruction set.

pub const STACK_MAX: usize = 256;

pub mod op {
    pub const HALT: u8 = 0x00;
    pub const SLEEP: u8 = 0x01;
    pub const MOVE: u8 = 0x02;
    pub const DIG: u8 = 0x03;
    pub const PLACE: u8 = 0x04;
    pub const EAT: u8 = 0x05;
    pub const SENSE: u8 = 0x06;
    pub const ENERGY: u8 = 0x07;
    pub const POP: u8 = 0x08;
    pub const DUP: u8 = 0x09;
    pub const PUSH: u8 = 0x0A;
    pub const JMP: u8 = 0x0B;
    pub const JZ: u8 = 0x0C;
    pub const JNZ: u8 = 0x0D;
    pub const EQ: u8 = 0x0E;
    pub const LT: u8 = 0x0F;
    pub const ADD: u8 = 0x10;
    pub const SUB: u8 = 0x11;
    pub const SUICIDE: u8 = 0x12;
}

pub mod dir {
    pub const N: u8 = 0;
    pub const E: u8 = 1;
    pub const S: u8 = 2;
    pub const W: u8 = 3;
}

pub mod tile {
    pub const EMPTY: i32 = 0;
    pub const SOLID: i32 = 1;
    pub const CREATURE: i32 = 2;
    pub const CORPSE: i32 = 3;
}
