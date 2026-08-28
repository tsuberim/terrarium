//! Closed-box world: mass ledger + deterministic fixed-point physics.

use std::collections::HashMap;
use std::fmt;

use crate::program::{Instr, Program, MAX_OPS_PER_TICK};

/// Playable torus size in fixed-point units. Center is (0, 0); coordinates wrap independently on each axis.
pub const WORLD_WIDTH: i32 = 800_000;
pub const WORLD_HEIGHT: i32 = 800_000;

/// Mass cost of one `sense` verb.
pub const SENSE_COST: u64 = 2;

/// Friction applied each tick (velocity *= (FRICTION_NUM/FRICTION_DEN)).
const FRICTION_NUM: i32 = 92;
const FRICTION_DEN: i32 = 100;

/// How many register slots a cell has (sense fills R0..R4).
const REGS: usize = 8;

/// A quantity of matter. The closed box is denominated in these units.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Mass(u64);

impl Mass {
    pub const ZERO: Self = Self(0);

    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub const fn saturating_add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    pub const fn checked_add(self, other: Self) -> Option<Self> {
        match self.0.checked_add(other.0) {
            Some(v) => Some(Self(v)),
            None => None,
        }
    }

    pub const fn checked_sub(self, other: Self) -> Option<Self> {
        match self.0.checked_sub(other.0) {
            Some(v) => Some(Self(v)),
            None => None,
        }
    }

    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl fmt::Display for Mass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Stable id of a living cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CellId(u64);

impl CellId {
    pub const fn get(self) -> u64 {
        self.0
    }

    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub(crate) const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }
}

/// Stable id of an inert dump (wall, shot, debris). Anyone can absorb it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InertId(u64);

impl InertId {
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KernelError {
    UnknownCell,
    UnknownInert,
    InsufficientMass,
    ZeroAmount,
    BadProgram,
}

impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownCell => write!(f, "unknown cell"),
            Self::UnknownInert => write!(f, "unknown inert mass"),
            Self::InsufficientMass => write!(f, "insufficient mass"),
            Self::ZeroAmount => write!(f, "amount must be greater than zero"),
            Self::BadProgram => write!(f, "bad program"),
        }
    }
}

impl std::error::Error for KernelError {}

struct Cell {
    mass: Mass,
    x: i32,
    y: i32,
    vx: i32,
    vy: i32,
    program: Program,
    pc: u16,
    regs: [i32; REGS],
    halted: bool,
    /// Impulse accumulated this tick from thrust verbs.
    impulse_x: i32,
    impulse_y: i32,
}

struct Inert {
    mass: Mass,
    x: i32,
    y: i32,
}

/// Read-only cell snapshot for the skin.
#[derive(Clone, Debug)]
pub struct CellView {
    pub id: CellId,
    pub mass: Mass,
    pub x: i32,
    pub y: i32,
    pub vx: i32,
    pub vy: i32,
    pub pc: u16,
    pub halted: bool,
}

/// Read-only inert snapshot for the skin.
#[derive(Clone, Debug)]
pub struct InertView {
    pub id: InertId,
    pub mass: Mass,
    pub x: i32,
    pub y: i32,
}

/// Full world snapshot (JSON-friendly fields via Display helpers on the WASM side).
#[derive(Clone, Debug)]
pub struct WorldSnapshot {
    pub tick: u64,
    pub total_mass: Mass,
    pub house_burned: Mass,
    pub spawned_mass: Mass,
    pub width: i32,
    pub height: i32,
    pub cells: Vec<CellView>,
    pub inert: Vec<InertView>,
}

/// The closed box.
pub struct World {
    next_cell: u64,
    next_inert: u64,
    cells: HashMap<u64, Cell>,
    inert: HashMap<u64, Inert>,
    house_burned: Mass,
    /// Cash-in total. Until cash-out exists: spawned_mass == total_mass + house_burned.
    spawned_mass: Mass,
    width: i32,
    height: i32,
    tick: u64,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self::with_size(WORLD_WIDTH, WORLD_HEIGHT)
    }

    pub fn with_size(width: i32, height: i32) -> Self {
        Self {
            next_cell: 0,
            next_inert: 0,
            cells: HashMap::new(),
            inert: HashMap::new(),
            house_burned: Mass::ZERO,
            spawned_mass: Mass::ZERO,
            width,
            height,
            tick: 0,
        }
    }

    pub fn tick_count(&self) -> u64 {
        self.tick
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    /// Living cells plus inert dumps. Does not include mass the house burned.
    pub fn total_mass(&self) -> Mass {
        let mut sum = Mass::ZERO;
        for cell in self.cells.values() {
            sum = sum.checked_add(cell.mass).expect("total mass overflow");
        }
        for dump in self.inert.values() {
            sum = sum.checked_add(dump.mass).expect("total mass overflow");
        }
        sum
    }

    /// Mass destroyed by acting and computing. Only ever increases.
    pub fn house_burned(&self) -> Mass {
        self.house_burned
    }

    /// Total mass ever spawned (cash-in). Cash-out does not exist yet.
    pub fn spawned_mass(&self) -> Mass {
        self.spawned_mass
    }

    pub fn cell_mass(&self, id: CellId) -> Option<Mass> {
        self.cells.get(&id.0).map(|c| c.mass)
    }

    pub fn inert_mass(&self, id: InertId) -> Option<Mass> {
        self.inert.get(&id.0).map(|i| i.mass)
    }

    pub fn cell_pos(&self, id: CellId) -> Option<(i32, i32)> {
        self.cells.get(&id.0).map(|c| (c.x, c.y))
    }

    /// Cash-in: new mass enters the box as a living cell at the origin.
    pub fn spawn_cell(&mut self, mass: Mass) -> Result<CellId, KernelError> {
        self.spawn_cell_at(mass, 0, 0)
    }

    /// Cash-in at a position. Coordinates wrap onto the torus.
    pub fn spawn_cell_at(&mut self, mass: Mass, x: i32, y: i32) -> Result<CellId, KernelError> {
        if mass.is_zero() {
            return Err(KernelError::ZeroAmount);
        }
        let (x, y) = wrap_position(x, y, self.width, self.height);
        let id = CellId(self.next_cell);
        self.next_cell += 1;
        self.cells.insert(
            id.0,
            Cell {
                mass,
                x,
                y,
                vx: 0,
                vy: 0,
                program: Program::default(),
                pc: 0,
                regs: [0; REGS],
                halted: false,
                impulse_x: 0,
                impulse_y: 0,
            },
        );
        self.spawned_mass = self
            .spawned_mass
            .checked_add(mass)
            .expect("spawned_mass overflow");
        Ok(id)
    }

    /// Install a guest program on a cell. Resets PC.
    pub fn set_program(&mut self, id: CellId, program: Program) -> Result<(), KernelError> {
        let cell = self.cells.get_mut(&id.0).ok_or(KernelError::UnknownCell)?;
        cell.program = program;
        cell.pc = 0;
        cell.halted = false;
        Ok(())
    }

    /// Burn mass from a cell to the house. Destroyed. Acting and computing cost this.
    pub fn spend(&mut self, id: CellId, amount: Mass) -> Result<(), KernelError> {
        if amount.is_zero() {
            return Err(KernelError::ZeroAmount);
        }
        let remaining = {
            let cell = self.cells.get_mut(&id.0).ok_or(KernelError::UnknownCell)?;
            cell.mass
                .checked_sub(amount)
                .ok_or(KernelError::InsufficientMass)?
        };
        self.house_burned = self
            .house_burned
            .checked_add(amount)
            .expect("house_burned overflow");
        if remaining.is_zero() {
            self.cells.remove(&id.0);
        } else {
            self.cells.get_mut(&id.0).unwrap().mass = remaining;
        }
        Ok(())
    }

    /// Dump inert matter (walls, shots, debris). Still in the box. Anyone can absorb it.
    pub fn dump_matter(&mut self, id: CellId, amount: Mass) -> Result<InertId, KernelError> {
        if amount.is_zero() {
            return Err(KernelError::ZeroAmount);
        }
        let (remaining, x, y) = {
            let cell = self.cells.get_mut(&id.0).ok_or(KernelError::UnknownCell)?;
            let remaining = cell
                .mass
                .checked_sub(amount)
                .ok_or(KernelError::InsufficientMass)?;
            (remaining, cell.x, cell.y)
        };
        if remaining.is_zero() {
            self.cells.remove(&id.0);
        } else {
            self.cells.get_mut(&id.0).unwrap().mass = remaining;
        }
        let inert_id = InertId(self.next_inert);
        self.next_inert += 1;
        let (x, y) = wrap_position(x, y, self.width, self.height);
        self.inert.insert(
            inert_id.0,
            Inert {
                mass: amount,
                x,
                y,
            },
        );
        Ok(inert_id)
    }

    /// Absorb is an explicit verb. Touching is not auto-eat.
    pub fn absorb_matter(
        &mut self,
        cell_id: CellId,
        inert_id: InertId,
        amount: Mass,
    ) -> Result<(), KernelError> {
        if amount.is_zero() {
            return Err(KernelError::ZeroAmount);
        }
        if !self.cells.contains_key(&cell_id.0) {
            return Err(KernelError::UnknownCell);
        }
        let inert = self
            .inert
            .get(&inert_id.0)
            .ok_or(KernelError::UnknownInert)?;
        let remaining_inert = inert
            .mass
            .checked_sub(amount)
            .ok_or(KernelError::InsufficientMass)?;
        {
            let cell = self.cells.get_mut(&cell_id.0).unwrap();
            cell.mass = cell.mass.checked_add(amount).expect("cell mass overflow");
        }
        if remaining_inert.is_zero() {
            self.inert.remove(&inert_id.0);
        } else {
            self.inert.get_mut(&inert_id.0).unwrap().mass = remaining_inert;
        }
        Ok(())
    }

    /// Thrust: apply an impulse and burn mass to the house.
    pub fn thrust(&mut self, id: CellId, fx: i32, fy: i32) -> Result<(), KernelError> {
        let cost = thrust_cost(fx, fy);
        self.spend(id, Mass::new(cost))?;
        if let Some(cell) = self.cells.get_mut(&id.0) {
            cell.impulse_x = cell.impulse_x.saturating_add(fx);
            cell.impulse_y = cell.impulse_y.saturating_add(fy);
        }
        Ok(())
    }

    /// Sense nearest other body. Fills registers. Burns mass.
    pub fn sense(&mut self, id: CellId) -> Result<(), KernelError> {
        self.spend(id, Mass::new(SENSE_COST))?;
        let Some(me) = self.cells.get(&id.0) else {
            // Died paying for sense.
            return Ok(());
        };
        let mx = me.x;
        let my = me.y;
        let mut best: Option<(i64, i32, i32, i32)> = None; // dist2, dx, dy, kind
        for (oid, other) in &self.cells {
            if *oid == id.0 {
                continue;
            }
            let dx = toroidal_delta(mx, other.x, self.width);
            let dy = toroidal_delta(my, other.y, self.height);
            let d2 = toroidal_dist2(mx, my, other.x, other.y, self.width, self.height);
            if best.map(|(b, _, _, _)| d2 < b).unwrap_or(true) {
                best = Some((d2, dx, dy, 1));
            }
        }
        for dump in self.inert.values() {
            let dx = toroidal_delta(mx, dump.x, self.width);
            let dy = toroidal_delta(my, dump.y, self.height);
            let d2 = toroidal_dist2(mx, my, dump.x, dump.y, self.width, self.height);
            if best.map(|(b, _, _, _)| d2 < b).unwrap_or(true) {
                best = Some((d2, dx, dy, 0));
            }
        }
        let cell = self.cells.get_mut(&id.0).unwrap();
        match best {
            Some((d2, dx, dy, kind)) => {
                cell.regs[0] = 1;
                cell.regs[1] = dx;
                cell.regs[2] = dy;
                cell.regs[3] = kind;
                cell.regs[4] = isqrt_i64(d2) as i32;
            }
            None => {
                cell.regs[0] = 0;
                cell.regs[1] = 0;
                cell.regs[2] = 0;
                cell.regs[3] = 0;
                cell.regs[4] = 0;
            }
        }
        Ok(())
    }

    /// Absorb nearest inert that is within reach. Explicit. Conserves total_mass.
    pub fn absorb_nearest(&mut self, id: CellId) -> Result<(), KernelError> {
        let (cx, cy, cmass) = {
            let cell = self.cells.get(&id.0).ok_or(KernelError::UnknownCell)?;
            (cell.x, cell.y, cell.mass)
        };
        let reach = body_radius(cmass).saturating_add(8_000);
        let mut best: Option<(i64, u64, Mass)> = None;
        for (iid, dump) in &self.inert {
            let d2 = toroidal_dist2(cx, cy, dump.x, dump.y, self.width, self.height);
            let limit = reach.saturating_add(body_radius(dump.mass));
            if d2 > (limit as i64) * (limit as i64) {
                continue;
            }
            if best.map(|(b, _, _)| d2 < b).unwrap_or(true) {
                best = Some((d2, *iid, dump.mass));
            }
        }
        let Some((_, inert_id, amount)) = best else {
            return Ok(());
        };
        self.absorb_matter(id, InertId(inert_id), amount)
    }

    /// Advance the world one tick: run programs, integrate motion. Deterministic.
    pub fn tick(&mut self) {
        let ids: Vec<u64> = {
            let mut v: Vec<u64> = self.cells.keys().copied().collect();
            v.sort_unstable();
            v
        };

        for id in &ids {
            if !self.cells.contains_key(id) {
                continue;
            }
            {
                let cell = self.cells.get_mut(id).unwrap();
                cell.impulse_x = 0;
                cell.impulse_y = 0;
            }
            self.run_cell_program(CellId(*id));
        }

        // Integrate after all programs so order of cells does not bias sensing mid-move.
        let ids: Vec<u64> = {
            let mut v: Vec<u64> = self.cells.keys().copied().collect();
            v.sort_unstable();
            v
        };
        for id in ids {
            let Some(cell) = self.cells.get_mut(&id) else {
                continue;
            };
            cell.vx = cell.vx.saturating_add(cell.impulse_x);
            cell.vy = cell.vy.saturating_add(cell.impulse_y);
            cell.vx = (cell.vx as i64 * FRICTION_NUM as i64 / FRICTION_DEN as i64) as i32;
            cell.vy = (cell.vy as i64 * FRICTION_NUM as i64 / FRICTION_DEN as i64) as i32;
            cell.x = cell.x.saturating_add(cell.vx);
            cell.y = cell.y.saturating_add(cell.vy);
            let (x, y) = wrap_position(cell.x, cell.y, self.width, self.height);
            cell.x = x;
            cell.y = y;
            cell.impulse_x = 0;
            cell.impulse_y = 0;
        }

        self.tick = self.tick.saturating_add(1);
    }

    fn run_cell_program(&mut self, id: CellId) {
        for _ in 0..MAX_OPS_PER_TICK {
            if !self.cells.contains_key(&id.0) {
                return;
            }
            let (halted, pc, op) = {
                let cell = self.cells.get(&id.0).unwrap();
                if cell.halted || cell.program.is_empty() {
                    return;
                }
                let pc = cell.pc as usize;
                if pc >= cell.program.ops.len() {
                    return;
                }
                (cell.halted, cell.pc, cell.program.ops[pc].clone())
            };
            if halted {
                return;
            }

            match op {
                Instr::Halt => {
                    let cell = self.cells.get_mut(&id.0).unwrap();
                    cell.halted = true;
                    return;
                }
                Instr::Sleep => {
                    let cell = self.cells.get_mut(&id.0).unwrap();
                    cell.pc = pc.saturating_add(1);
                    return;
                }
                Instr::Thrust { fx, fy } => {
                    let _ = self.thrust(id, fx as i32, fy as i32);
                    if let Some(cell) = self.cells.get_mut(&id.0) {
                        cell.pc = pc.saturating_add(1);
                    } else {
                        return;
                    }
                }
                Instr::ThrustToward { mag } => {
                    let (dx, dy) = {
                        let cell = self.cells.get(&id.0).unwrap();
                        (cell.regs[1], cell.regs[2])
                    };
                    let (fx, fy) = scale_toward(dx, dy, mag as i32);
                    let _ = self.thrust(id, fx, fy);
                    if let Some(cell) = self.cells.get_mut(&id.0) {
                        cell.pc = pc.saturating_add(1);
                    } else {
                        return;
                    }
                }
                Instr::Sense => {
                    let _ = self.sense(id);
                    if let Some(cell) = self.cells.get_mut(&id.0) {
                        cell.pc = pc.saturating_add(1);
                    } else {
                        return;
                    }
                }
                Instr::Absorb => {
                    let _ = self.absorb_nearest(id);
                    if let Some(cell) = self.cells.get_mut(&id.0) {
                        cell.pc = pc.saturating_add(1);
                    } else {
                        return;
                    }
                }
                Instr::Dump { amount } => {
                    let _ = self.dump_matter(id, Mass::new(amount as u64));
                    if let Some(cell) = self.cells.get_mut(&id.0) {
                        cell.pc = pc.saturating_add(1);
                    } else {
                        return;
                    }
                }
                Instr::Jump { addr } => {
                    let cell = self.cells.get_mut(&id.0).unwrap();
                    cell.pc = addr;
                }
                Instr::Jnz { reg, addr } => {
                    let cell = self.cells.get_mut(&id.0).unwrap();
                    let r = cell.regs.get(reg as usize).copied().unwrap_or(0);
                    if r != 0 {
                        cell.pc = addr;
                    } else {
                        cell.pc = pc.saturating_add(1);
                    }
                }
                Instr::Jz { reg, addr } => {
                    let cell = self.cells.get_mut(&id.0).unwrap();
                    let r = cell.regs.get(reg as usize).copied().unwrap_or(0);
                    if r == 0 {
                        cell.pc = addr;
                    } else {
                        cell.pc = pc.saturating_add(1);
                    }
                }
            }
        }
    }

    pub fn snapshot(&self) -> WorldSnapshot {
        let mut cells: Vec<CellView> = self
            .cells
            .iter()
            .map(|(id, c)| CellView {
                id: CellId(*id),
                mass: c.mass,
                x: c.x,
                y: c.y,
                vx: c.vx,
                vy: c.vy,
                pc: c.pc,
                halted: c.halted,
            })
            .collect();
        cells.sort_by_key(|c| c.id.0);
        let mut inert: Vec<InertView> = self
            .inert
            .iter()
            .map(|(id, i)| InertView {
                id: InertId(*id),
                mass: i.mass,
                x: i.x,
                y: i.y,
            })
            .collect();
        inert.sort_by_key(|i| i.id.0);
        WorldSnapshot {
            tick: self.tick,
            total_mass: self.total_mass(),
            house_burned: self.house_burned,
            spawned_mass: self.spawned_mass,
            width: self.width,
            height: self.height,
            cells,
            inert,
        }
    }
}

fn thrust_cost(fx: i32, fy: i32) -> u64 {
    let effort = (fx.unsigned_abs() as u64).saturating_add(fy.unsigned_abs() as u64);
    // ~1 mass per 25 impulse units, minimum 1 so every thrust burns.
    std::cmp::max(1, effort / 25)
}

fn body_radius(mass: Mass) -> i32 {
    // sqrt-ish: more mass → larger blob. Floor high enough to see on the skin canvas.
    let m = mass.get();
    let r = isqrt(m.saturating_mul(2_000)) as i32;
    std::cmp::max(4_000, std::cmp::min(r, 22_000))
}

fn wrap_axis(value: i32, extent: i32) -> i32 {
    if extent <= 0 {
        return 0;
    }
    let half = extent / 2;
    let mut v = value + half;
    v = v.rem_euclid(extent);
    v - half
}

fn wrap_position(x: i32, y: i32, width: i32, height: i32) -> (i32, i32) {
    (
        wrap_axis(x, width),
        wrap_axis(y, height),
    )
}

/// Shortest signed delta along one toroidal axis.
fn toroidal_delta(from: i32, to: i32, extent: i32) -> i32 {
    let mut d = to - from;
    let half = extent / 2;
    if d > half {
        d -= extent;
    } else if d < -half {
        d += extent;
    }
    d
}

fn toroidal_dist2(x1: i32, y1: i32, x2: i32, y2: i32, width: i32, height: i32) -> i64 {
    let dx = toroidal_delta(x1, x2, width);
    let dy = toroidal_delta(y1, y2, height);
    dist2(dx, dy)
}

fn dist2(dx: i32, dy: i32) -> i64 {
    let dx = dx as i64;
    let dy = dy as i64;
    dx * dx + dy * dy
}

fn isqrt(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

fn isqrt_i64(n: i64) -> i64 {
    if n <= 0 {
        return 0;
    }
    isqrt(n as u64) as i64
}

fn scale_toward(dx: i32, dy: i32, mag: i32) -> (i32, i32) {
    if mag == 0 || (dx == 0 && dy == 0) {
        return (0, 0);
    }
    let dist = isqrt_i64(dist2(dx, dy)).max(1);
    let fx = ((dx as i64) * (mag as i64) / dist) as i32;
    let fy = ((dy as i64) * (mag as i64) / dist) as i32;
    (fx, fy)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::program::{compile_text, Instr, Program};

    /// Closed ledger: cash-in equals mass still in the box plus house burn.
    fn assert_ledger(w: &World) {
        assert_eq!(
            w.spawned_mass().get(),
            w.total_mass().get() + w.house_burned().get(),
            "ledger: spawned={} total={} burned={}",
            w.spawned_mass(),
            w.total_mass(),
            w.house_burned()
        );
    }

    fn fingerprint(w: &World) -> Vec<(u64, i32, i32, u64, u16, bool)> {
        w.snapshot()
            .cells
            .iter()
            .map(|c| (c.id.get(), c.x, c.y, c.mass.get(), c.pc, c.halted))
            .collect()
    }

    #[test]
    fn spawn_adds_to_total_mass() {
        let mut w = World::new();
        assert_eq!(w.total_mass(), Mass::ZERO);
        assert_eq!(w.house_burned(), Mass::ZERO);
        assert_eq!(w.spawned_mass(), Mass::ZERO);
        w.spawn_cell(Mass::new(100)).unwrap();
        assert_eq!(w.total_mass(), Mass::new(100));
        assert_eq!(w.house_burned(), Mass::ZERO);
        assert_eq!(w.spawned_mass(), Mass::new(100));
        assert_ledger(&w);
    }

    #[test]
    fn dump_and_absorb_conserve_mass() {
        let mut w = World::new();
        let a = w.spawn_cell(Mass::new(50)).unwrap();
        let b = w.spawn_cell(Mass::new(30)).unwrap();
        let before = w.total_mass();
        let burned = w.house_burned();
        let dump = w.dump_matter(a, Mass::new(20)).unwrap();
        assert_eq!(w.total_mass(), before);
        assert_eq!(w.house_burned(), burned);
        assert_eq!(w.cell_mass(a), Some(Mass::new(30)));
        assert_eq!(w.inert_mass(dump), Some(Mass::new(20)));
        w.absorb_matter(b, dump, Mass::new(20)).unwrap();
        assert_eq!(w.total_mass(), before);
        assert_eq!(w.house_burned(), burned);
        assert_eq!(w.cell_mass(b), Some(Mass::new(50)));
        assert_eq!(w.inert_mass(dump), None);
        assert_ledger(&w);
    }

    #[test]
    fn spend_burns_to_house_and_is_the_only_leak() {
        let mut w = World::new();
        let c = w.spawn_cell(Mass::new(100)).unwrap();
        let dump = w.dump_matter(c, Mass::new(10)).unwrap();
        let closed = w.total_mass();
        assert_eq!(closed, Mass::new(100));
        w.spend(c, Mass::new(15)).unwrap();
        assert_eq!(w.total_mass(), Mass::new(closed.get() - 15));
        assert_eq!(w.house_burned(), Mass::new(15));
        let before = w.total_mass();
        w.absorb_matter(c, dump, Mass::new(10)).unwrap();
        assert_eq!(w.total_mass(), before);
        assert_eq!(w.house_burned(), Mass::new(15));
        assert_ledger(&w);
    }

    #[test]
    fn cannot_spend_more_than_cell_has() {
        let mut w = World::new();
        let c = w.spawn_cell(Mass::new(10)).unwrap();
        assert_eq!(
            w.spend(c, Mass::new(11)),
            Err(KernelError::InsufficientMass)
        );
        assert_eq!(w.total_mass(), Mass::new(10));
        assert_eq!(w.house_burned(), Mass::ZERO);
        assert_ledger(&w);
    }

    #[test]
    fn spend_to_zero_kills_the_cell() {
        let mut w = World::new();
        let c = w.spawn_cell(Mass::new(7)).unwrap();
        w.spend(c, Mass::new(7)).unwrap();
        assert_eq!(w.cell_mass(c), None);
        assert_eq!(w.total_mass(), Mass::ZERO);
        assert_eq!(w.house_burned(), Mass::new(7));
        assert_eq!(w.spend(c, Mass::new(1)), Err(KernelError::UnknownCell));
        assert_ledger(&w);
    }

    #[test]
    fn dump_to_zero_removes_cell_and_leaves_inert() {
        let mut w = World::new();
        let c = w.spawn_cell(Mass::new(12)).unwrap();
        let dump = w.dump_matter(c, Mass::new(12)).unwrap();
        assert_eq!(w.cell_mass(c), None);
        assert_eq!(w.inert_mass(dump), Some(Mass::new(12)));
        assert_eq!(w.total_mass(), Mass::new(12));
        assert_eq!(w.house_burned(), Mass::ZERO);
        assert_ledger(&w);
    }

    #[test]
    fn zero_amounts_are_rejected() {
        let mut w = World::new();
        assert_eq!(w.spawn_cell(Mass::ZERO), Err(KernelError::ZeroAmount));
        let c = w.spawn_cell(Mass::new(5)).unwrap();
        assert_eq!(w.spend(c, Mass::ZERO), Err(KernelError::ZeroAmount));
        assert_eq!(w.dump_matter(c, Mass::ZERO), Err(KernelError::ZeroAmount));
        let dump = w.dump_matter(c, Mass::new(2)).unwrap();
        assert_eq!(
            w.absorb_matter(c, dump, Mass::ZERO),
            Err(KernelError::ZeroAmount)
        );
        assert_eq!(w.total_mass(), Mass::new(5));
        assert_ledger(&w);
    }

    #[test]
    fn partial_absorb_leaves_inert_remainder() {
        let mut w = World::new();
        let a = w.spawn_cell(Mass::new(8)).unwrap();
        let b = w.spawn_cell(Mass::new(1)).unwrap();
        let dump = w.dump_matter(a, Mass::new(6)).unwrap();
        w.absorb_matter(b, dump, Mass::new(2)).unwrap();
        assert_eq!(w.inert_mass(dump), Some(Mass::new(4)));
        assert_eq!(w.cell_mass(b), Some(Mass::new(3)));
        assert_eq!(w.total_mass(), Mass::new(9));
        assert_ledger(&w);
    }

    #[test]
    fn closed_ledger_holds_across_mixed_ops() {
        let mut w = World::new();
        let a = w.spawn_cell_at(Mass::new(200), -5_000, 0).unwrap();
        let b = w.spawn_cell_at(Mass::new(150), 5_000, 0).unwrap();
        assert_ledger(&w);
        let dump = w.dump_matter(a, Mass::new(40)).unwrap();
        assert_ledger(&w);
        w.spend(b, Mass::new(10)).unwrap();
        assert_ledger(&w);
        w.absorb_matter(b, dump, Mass::new(20)).unwrap();
        assert_ledger(&w);
        w.thrust(a, 80, -40).unwrap();
        assert_ledger(&w);
        w.sense(b).unwrap();
        assert_ledger(&w);
        w.set_program(
            a,
            Program::new(vec![
                Instr::Thrust { fx: 30, fy: 10 },
                Instr::Sleep,
                Instr::Jump { addr: 0 },
            ]),
        )
        .unwrap();
        for _ in 0..25 {
            w.tick();
            assert_ledger(&w);
        }
    }

    #[test]
    fn house_burned_is_monotonic() {
        let mut w = World::new();
        let c = w.spawn_cell(Mass::new(500)).unwrap();
        let mut prev = w.house_burned().get();
        w.spend(c, Mass::new(3)).unwrap();
        assert!(w.house_burned().get() >= prev);
        prev = w.house_burned().get();
        w.thrust(c, 100, 0).unwrap();
        assert!(w.house_burned().get() >= prev);
        prev = w.house_burned().get();
        w.sense(c).unwrap();
        assert!(w.house_burned().get() >= prev);
        prev = w.house_burned().get();
        let dump = w.dump_matter(c, Mass::new(5)).unwrap();
        assert_eq!(w.house_burned().get(), prev);
        w.absorb_matter(c, dump, Mass::new(5)).unwrap();
        assert_eq!(w.house_burned().get(), prev);
        w.set_program(c, Program::new(vec![Instr::Sleep, Instr::Jump { addr: 0 }]))
            .unwrap();
        for _ in 0..10 {
            w.tick();
            assert_eq!(w.house_burned().get(), prev);
        }
        assert_ledger(&w);
    }

    #[test]
    fn dump_absorb_never_change_total_or_burn() {
        let mut w = World::new();
        let a = w.spawn_cell(Mass::new(80)).unwrap();
        let b = w.spawn_cell(Mass::new(40)).unwrap();
        let total = w.total_mass();
        let burned = w.house_burned();
        let dump = w.dump_matter(a, Mass::new(25)).unwrap();
        assert_eq!(w.total_mass(), total);
        assert_eq!(w.house_burned(), burned);
        w.absorb_matter(b, dump, Mass::new(10)).unwrap();
        assert_eq!(w.total_mass(), total);
        assert_eq!(w.house_burned(), burned);
        w.absorb_matter(b, dump, Mass::new(15)).unwrap();
        assert_eq!(w.total_mass(), total);
        assert_eq!(w.house_burned(), burned);
        assert_ledger(&w);
    }

    #[test]
    fn sleep_empty_halt_free_across_ticks() {
        let mut sleep_w = World::new();
        let s = sleep_w.spawn_cell(Mass::new(50)).unwrap();
        sleep_w
            .set_program(
                s,
                Program::new(vec![Instr::Sleep, Instr::Jump { addr: 0 }]),
            )
            .unwrap();

        let mut empty_w = World::new();
        let e = empty_w.spawn_cell(Mass::new(50)).unwrap();
        empty_w.set_program(e, Program::new(vec![])).unwrap();

        let mut halt_w = World::new();
        let h = halt_w.spawn_cell(Mass::new(50)).unwrap();
        halt_w
            .set_program(h, Program::new(vec![Instr::Halt]))
            .unwrap();

        for w in [&mut sleep_w, &mut empty_w, &mut halt_w] {
            for _ in 0..40 {
                w.tick();
            }
            assert_eq!(w.total_mass(), Mass::new(50));
            assert_eq!(w.house_burned(), Mass::ZERO);
            assert_ledger(w);
        }
        assert_eq!(sleep_w.cell_mass(s), Some(Mass::new(50)));
        assert_eq!(empty_w.cell_mass(e), Some(Mass::new(50)));
        assert_eq!(halt_w.cell_mass(h), Some(Mass::new(50)));
    }

    #[test]
    fn tick_determinism_identical_snapshots() {
        fn run() -> (Vec<(u64, i32, i32, u64, u16, bool)>, u64, u64) {
            let mut w = World::new();
            let a = w.spawn_cell_at(Mass::new(400), -10_000, 5_000).unwrap();
            let b = w.spawn_cell_at(Mass::new(300), 8_000, -4_000).unwrap();
            w.set_program(
                a,
                compile_text("thrust 60 20\nsleep\nthrust -40 30\nsleep\njump 0").unwrap(),
            )
            .unwrap();
            w.set_program(
                b,
                compile_text("sense\nthrust_toward 50\nsleep\njump 0").unwrap(),
            )
            .unwrap();
            for _ in 0..40 {
                w.tick();
            }
            let cells = fingerprint(&w);
            (cells, w.house_burned().get(), w.spawned_mass().get())
        }
        assert_eq!(run(), run());
    }

    #[test]
    fn wrap_axis_crosses_right_edge() {
        let w = WORLD_WIDTH;
        let half = w / 2;
        assert_eq!(wrap_axis(half - 1, w), half - 1);
        assert_eq!(wrap_axis(half, w), -half);
        assert_eq!(wrap_axis(half + 1, w), -half + 1);
    }

    #[test]
    fn wrap_axis_crosses_left_edge() {
        let w = WORLD_WIDTH;
        let half = w / 2;
        assert_eq!(wrap_axis(-half, w), -half);
        assert_eq!(wrap_axis(-half - 1, w), half - 1);
    }

    #[test]
    fn toroidal_delta_prefers_shortest_path() {
        let w = WORLD_WIDTH;
        let half = w / 2;
        assert_eq!(toroidal_delta(half - 10_000, -half + 5_000, w), 15_000);
        assert_eq!(toroidal_delta(-half + 5_000, half - 10_000, w), -15_000);
    }

    #[test]
    fn toroidal_dist2_matches_wrap_around() {
        let w = WORLD_WIDTH;
        let h = WORLD_HEIGHT;
        let half_w = w / 2;
        let near = toroidal_dist2(half_w - 1_000, 0, -half_w + 500, 0, w, h);
        let direct = dist2(-1_500, 0);
        assert_eq!(near, direct);
        let far = toroidal_dist2(0, 0, half_w - 1, 0, w, h);
        assert_eq!(far, dist2(half_w - 1, 0));
    }

    #[test]
    fn thrust_wraps_cell_across_x_edge() {
        let mut w = World::new();
        let half = w.width() / 2;
        let c = w.spawn_cell_at(Mass::new(1_000), half - 500, 0).unwrap();
        w.set_program(
            c,
            Program::new(vec![
                Instr::Thrust { fx: 2000, fy: 0 },
                Instr::Sleep,
                Instr::Jump { addr: 0 },
            ]),
        )
        .unwrap();
        for _ in 0..80 {
            w.tick();
            assert_ledger(&w);
        }
        let (x, y) = w.cell_pos(c).unwrap();
        assert!(x < half, "cell should wrap to left side, got x={x}");
        assert_eq!(y, 0);
    }

    #[test]
    fn thrust_wraps_cell_across_y_edge() {
        let mut w = World::new();
        let half = w.height() / 2;
        let c = w.spawn_cell_at(Mass::new(1_000), 0, half - 500).unwrap();
        w.set_program(
            c,
            Program::new(vec![
                Instr::Thrust { fx: 0, fy: 2000 },
                Instr::Sleep,
                Instr::Jump { addr: 0 },
            ]),
        )
        .unwrap();
        for _ in 0..80 {
            w.tick();
            assert_ledger(&w);
        }
        let (x, y) = w.cell_pos(c).unwrap();
        assert_eq!(x, 0);
        assert!(y < half, "cell should wrap to bottom side, got y={y}");
    }

    #[test]
    fn sense_uses_toroidal_nearest() {
        let mut w = World::new();
        let half = w.width() / 2;
        let observer = w.spawn_cell_at(Mass::new(5000), -half + 2_000, 0).unwrap();
        let _target = w.spawn_cell_at(Mass::new(200), half - 5_000, 0).unwrap();
        w.set_program(
            observer,
            compile_text("sense\nthrust_toward 100\nsleep\njump 0").unwrap(),
        )
        .unwrap();
        let (x_before, _) = w.cell_pos(observer).unwrap();
        w.tick();
        let (x_after, _) = w.cell_pos(observer).unwrap();
        assert!(
            x_after < x_before,
            "sense should pick wrapped target leftward (before={x_before}, after={x_after})"
        );
        assert_ledger(&w);
    }

    #[test]
    fn sense_costs_sense_cost_and_cannot_mint() {
        let mut w = World::new();
        let c = w.spawn_cell(Mass::new(SENSE_COST + 5)).unwrap();
        let before_total = w.total_mass();
        let before_burn = w.house_burned();
        w.sense(c).unwrap();
        assert_eq!(w.house_burned().get(), before_burn.get() + SENSE_COST);
        assert_eq!(w.total_mass().get(), before_total.get() - SENSE_COST);
        assert_ledger(&w);

        // Spend down to less than SENSE_COST.
        let left = w.cell_mass(c).unwrap().get();
        assert!(left < SENSE_COST || left >= 1);
        if left > 0 {
            w.spend(c, Mass::new(left)).unwrap();
        }
        // Cell may be gone; spawn a tiny cell that cannot afford sense.
        let poor = w.spawn_cell(Mass::new(1)).unwrap();
        let total = w.total_mass();
        let burned = w.house_burned();
        let spawned = w.spawned_mass();
        assert_eq!(
            w.sense(poor),
            Err(KernelError::InsufficientMass)
        );
        assert_eq!(w.total_mass(), total);
        assert_eq!(w.house_burned(), burned);
        assert_eq!(w.spawned_mass(), spawned);
        assert_eq!(w.cell_mass(poor), Some(Mass::new(1)));
        assert_ledger(&w);
    }

    #[test]
    fn tick_thrust_moves_cell_and_burns_mass() {
        let mut w = World::new();
        let c = w.spawn_cell_at(Mass::new(1_000), 0, 0).unwrap();
        w.set_program(
            c,
            Program::new(vec![
                Instr::Thrust { fx: 500, fy: 0 },
                Instr::Sleep,
                Instr::Jump { addr: 0 },
            ]),
        )
        .unwrap();
        let before_mass = w.total_mass();
        let before_burn = w.house_burned();
        w.tick();
        let (x, y) = w.cell_pos(c).unwrap();
        assert!(x > 0, "cell should move +x after thrust, got ({x},{y})");
        assert_eq!(y, 0);
        assert!(w.total_mass().get() < before_mass.get());
        assert!(w.house_burned().get() > before_burn.get());
        assert_eq!(
            before_mass.get() - w.total_mass().get(),
            w.house_burned().get() - before_burn.get()
        );
        assert_ledger(&w);
    }

    #[test]
    fn sleep_is_free_across_ticks() {
        let mut w = World::new();
        let c = w.spawn_cell(Mass::new(50)).unwrap();
        w.set_program(
            c,
            Program::new(vec![Instr::Sleep, Instr::Jump { addr: 0 }]),
        )
        .unwrap();
        for _ in 0..20 {
            w.tick();
        }
        assert_eq!(w.cell_mass(c), Some(Mass::new(50)));
        assert_eq!(w.house_burned(), Mass::ZERO);
        assert_eq!(w.total_mass(), Mass::new(50));
        assert_ledger(&w);
    }

    #[test]
    fn absorb_nearest_conserves_and_is_explicit() {
        let mut w = World::new();
        let a = w.spawn_cell_at(Mass::new(100), 0, 0).unwrap();
        let dump = w.dump_matter(a, Mass::new(30)).unwrap();
        // Sitting on the dump; without absorb verb, inert stays.
        w.tick();
        assert_eq!(w.inert_mass(dump), Some(Mass::new(30)));
        let before = w.total_mass();
        w.absorb_nearest(a).unwrap();
        assert_eq!(w.total_mass(), before);
        assert_eq!(w.inert_mass(dump), None);
        assert_eq!(w.cell_mass(a), Some(Mass::new(100)));
        assert_ledger(&w);
    }

    #[test]
    fn chase_program_senses_and_spends() {
        let mut w = World::new();
        let hunter = w.spawn_cell_at(Mass::new(500), -20_000, 0).unwrap();
        let _prey = w.spawn_cell_at(Mass::new(200), 20_000, 0).unwrap();
        let prog = compile_text(
            r#"
            sense
            jnz 0 4
            sleep
            jump 0
            thrust_toward 80
            sleep
            jump 0
        "#,
        )
        .unwrap();
        w.set_program(hunter, prog).unwrap();
        let before = w.house_burned();
        w.tick();
        assert!(w.house_burned().get() > before.get());
        let (x, _) = w.cell_pos(hunter).unwrap();
        assert!(x > -20_000, "hunter x={x}");
        assert_ledger(&w);
    }

    #[test]
    fn tick_is_deterministic() {
        fn run() -> Vec<(i32, i32, u64)> {
            let mut w = World::new();
            let a = w.spawn_cell_at(Mass::new(400), -10_000, 5_000).unwrap();
            let b = w.spawn_cell_at(Mass::new(300), 8_000, -4_000).unwrap();
            w.set_program(
                a,
                compile_text("thrust 60 20\nsleep\nthrust -40 30\nsleep\njump 0").unwrap(),
            )
            .unwrap();
            w.set_program(
                b,
                compile_text("sense\nthrust_toward 50\nsleep\njump 0").unwrap(),
            )
            .unwrap();
            for _ in 0..40 {
                w.tick();
            }
            let mut out = Vec::new();
            for c in w.snapshot().cells {
                out.push((c.x, c.y, c.mass.get()));
            }
            out.push((0, 0, w.house_burned().get()));
            out
        }
        assert_eq!(run(), run());
    }
}
