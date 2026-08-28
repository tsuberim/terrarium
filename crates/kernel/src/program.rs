//! Tiny guest bytecode for cell programs.
//!
//! Skin can paste a line-oriented text program; the kernel compiles it.
//! Verbs: thrust, sense, absorb, dump. Sleep is free. Jump for loops.

use crate::world::KernelError;

/// Soft cap on how many ops a cell may run in one tick (compute is fuel).
pub const MAX_OPS_PER_TICK: u32 = 32;

/// One instruction in a cell's program.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Instr {
    /// Stop the program permanently (PC stays; further ticks no-op).
    Halt,
    /// End this tick; keep PC. Free.
    Sleep,
    /// Apply thrust impulse this tick. Costs mass.
    Thrust { fx: i16, fy: i16 },
    /// Thrust toward last sense hit using registers R1/R2, magnitude `mag`.
    ThrustToward { mag: u16 },
    /// Sense nearest other body. Fills registers. Costs mass.
    Sense,
    /// Absorb nearest in-range inert dump (explicit verb). Conserves total_mass.
    Absorb,
    /// Dump inert mass at the cell's position. Conserves total_mass.
    Dump { amount: u32 },
    /// Unconditional jump to instruction index.
    Jump { addr: u16 },
    /// Jump if register `reg` is non-zero.
    Jnz { reg: u8, addr: u16 },
    /// Jump if register `reg` is zero.
    Jz { reg: u8, addr: u16 },
}

/// A compiled guest program.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Program {
    pub ops: Vec<Instr>,
}

impl Program {
    pub fn new(ops: Vec<Instr>) -> Self {
        Self { ops }
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }
}

/// Encode a program to a compact byte buffer (portable across skin ↔ WASM).
pub fn encode_program(program: &Program) -> Vec<u8> {
    let mut out = Vec::new();
    for op in &program.ops {
        match *op {
            Instr::Halt => out.push(0),
            Instr::Sleep => out.push(1),
            Instr::Thrust { fx, fy } => {
                out.push(2);
                out.extend_from_slice(&fx.to_le_bytes());
                out.extend_from_slice(&fy.to_le_bytes());
            }
            Instr::ThrustToward { mag } => {
                out.push(3);
                out.extend_from_slice(&mag.to_le_bytes());
            }
            Instr::Sense => out.push(4),
            Instr::Absorb => out.push(5),
            Instr::Dump { amount } => {
                out.push(6);
                out.extend_from_slice(&amount.to_le_bytes());
            }
            Instr::Jump { addr } => {
                out.push(7);
                out.extend_from_slice(&addr.to_le_bytes());
            }
            Instr::Jnz { reg, addr } => {
                out.push(8);
                out.push(reg);
                out.extend_from_slice(&addr.to_le_bytes());
            }
            Instr::Jz { reg, addr } => {
                out.push(9);
                out.push(reg);
                out.extend_from_slice(&addr.to_le_bytes());
            }
        }
    }
    out
}

/// Decode a byte buffer into a program.
pub fn decode_program(bytes: &[u8]) -> Result<Program, KernelError> {
    let mut ops = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let op = bytes[i];
        i += 1;
        match op {
            0 => ops.push(Instr::Halt),
            1 => ops.push(Instr::Sleep),
            2 => {
                let fx = read_i16(bytes, &mut i)?;
                let fy = read_i16(bytes, &mut i)?;
                ops.push(Instr::Thrust { fx, fy });
            }
            3 => {
                let mag = read_u16(bytes, &mut i)?;
                ops.push(Instr::ThrustToward { mag });
            }
            4 => ops.push(Instr::Sense),
            5 => ops.push(Instr::Absorb),
            6 => {
                let amount = read_u32(bytes, &mut i)?;
                ops.push(Instr::Dump { amount });
            }
            7 => {
                let addr = read_u16(bytes, &mut i)?;
                ops.push(Instr::Jump { addr });
            }
            8 => {
                let reg = read_u8(bytes, &mut i)?;
                let addr = read_u16(bytes, &mut i)?;
                ops.push(Instr::Jnz { reg, addr });
            }
            9 => {
                let reg = read_u8(bytes, &mut i)?;
                let addr = read_u16(bytes, &mut i)?;
                ops.push(Instr::Jz { reg, addr });
            }
            _ => return Err(KernelError::BadProgram),
        }
    }
    Ok(Program::new(ops))
}

/// Compile a line-oriented text program.
///
/// Lines: `halt`, `sleep`, `thrust <fx> <fy>`, `thrust_toward <mag>`,
/// `sense`, `absorb`, `dump <amount>`, `jump <addr>`, `jnz <reg> <addr>`,
/// `jz <reg> <addr>`. `#` comments and blank lines ignored.
pub fn compile_text(src: &str) -> Result<Program, KernelError> {
    let mut ops = Vec::new();
    for raw in src.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let verb = parts.next().ok_or(KernelError::BadProgram)?;
        let op = match verb {
            "halt" => Instr::Halt,
            "sleep" => Instr::Sleep,
            "thrust" => {
                let fx: i16 = parse_next(&mut parts)?;
                let fy: i16 = parse_next(&mut parts)?;
                Instr::Thrust { fx, fy }
            }
            "thrust_toward" => {
                let mag: u16 = parse_next(&mut parts)?;
                Instr::ThrustToward { mag }
            }
            "sense" => Instr::Sense,
            "absorb" => Instr::Absorb,
            "dump" => {
                let amount: u32 = parse_next(&mut parts)?;
                Instr::Dump { amount }
            }
            "jump" => {
                let addr: u16 = parse_next(&mut parts)?;
                Instr::Jump { addr }
            }
            "jnz" => {
                let reg: u8 = parse_next(&mut parts)?;
                let addr: u16 = parse_next(&mut parts)?;
                Instr::Jnz { reg, addr }
            }
            "jz" => {
                let reg: u8 = parse_next(&mut parts)?;
                let addr: u16 = parse_next(&mut parts)?;
                Instr::Jz { reg, addr }
            }
            _ => return Err(KernelError::BadProgram),
        };
        if parts.next().is_some() {
            return Err(KernelError::BadProgram);
        }
        ops.push(op);
    }
    Ok(Program::new(ops))
}

fn parse_next<'a, T: std::str::FromStr>(
    parts: &mut impl Iterator<Item = &'a str>,
) -> Result<T, KernelError> {
    parts
        .next()
        .ok_or(KernelError::BadProgram)?
        .parse()
        .map_err(|_| KernelError::BadProgram)
}

fn read_u8(bytes: &[u8], i: &mut usize) -> Result<u8, KernelError> {
    if *i >= bytes.len() {
        return Err(KernelError::BadProgram);
    }
    let v = bytes[*i];
    *i += 1;
    Ok(v)
}

fn read_i16(bytes: &[u8], i: &mut usize) -> Result<i16, KernelError> {
    if *i + 2 > bytes.len() {
        return Err(KernelError::BadProgram);
    }
    let v = i16::from_le_bytes([bytes[*i], bytes[*i + 1]]);
    *i += 2;
    Ok(v)
}

fn read_u16(bytes: &[u8], i: &mut usize) -> Result<u16, KernelError> {
    if *i + 2 > bytes.len() {
        return Err(KernelError::BadProgram);
    }
    let v = u16::from_le_bytes([bytes[*i], bytes[*i + 1]]);
    *i += 2;
    Ok(v)
}

fn read_u32(bytes: &[u8], i: &mut usize) -> Result<u32, KernelError> {
    if *i + 4 > bytes.len() {
        return Err(KernelError::BadProgram);
    }
    let v = u32::from_le_bytes([bytes[*i], bytes[*i + 1], bytes[*i + 2], bytes[*i + 3]]);
    *i += 4;
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_encode_decode() {
        let p = Program::new(vec![
            Instr::Sense,
            Instr::Jnz { reg: 0, addr: 3 },
            Instr::Thrust { fx: 10, fy: -4 },
            Instr::ThrustToward { mag: 40 },
            Instr::Absorb,
            Instr::Dump { amount: 5 },
            Instr::Jump { addr: 0 },
            Instr::Sleep,
            Instr::Halt,
        ]);
        let bytes = encode_program(&p);
        assert_eq!(decode_program(&bytes).unwrap(), p);
    }

    #[test]
    fn compile_wander_text() {
        let src = r#"
            # wander loop
            thrust 40 20
            sleep
            thrust -30 40
            sleep
            jump 0
        "#;
        let p = compile_text(src).unwrap();
        assert_eq!(p.ops.len(), 5);
        assert_eq!(p.ops[0], Instr::Thrust { fx: 40, fy: 20 });
        assert_eq!(p.ops[4], Instr::Jump { addr: 0 });
    }
}
