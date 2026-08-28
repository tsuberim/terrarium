//! Terrarium kernel: a closed box of matter.
//!
//! Creatures are blobs of mass with a program inside. Acting and computing
//! burn mass to the house (destroyed). Sleep is free. Conservation is
//! load-bearing: every gram is accounted for.
//!
//! This crate compiles natively and to WASM. The web skin hosts the same
//! kernel. Guest programs are a tiny bytecode ISA (thrust / sense / absorb /
//! dump). Physics is deterministic fixed-point 2D.

mod program;
mod world;
mod wasm_api;

pub use program::{compile_text, decode_program, encode_program, Instr, Program, MAX_OPS_PER_TICK};
pub use wasm_api::JsWorld;
pub use world::{
    CellId, CellView, InertId, InertView, KernelError, Mass, World, WorldSnapshot, SENSE_COST,
    WORLD_RADIUS,
};
