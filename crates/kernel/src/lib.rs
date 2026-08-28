//! Terrarium kernel: a closed box of matter.
//!
//! Creatures are blobs of mass with a program inside. Acting and computing
//! burn mass to the house (destroyed). Sleep is free. Conservation is
//! load-bearing: every gram is accounted for.
//!
//! v1 of this crate compiles natively and will compile to WASM. Later the
//! same kernel is the multiplayer server. Guest programs are WASM; mass is
//! their fuel. Physics will be deterministic fixed-point 2D; this crate
//! currently owns mass accounting.

use std::collections::HashMap;
use std::fmt;

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
}

impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownCell => write!(f, "unknown cell"),
            Self::UnknownInert => write!(f, "unknown inert mass"),
            Self::InsufficientMass => write!(f, "insufficient mass"),
            Self::ZeroAmount => write!(f, "amount must be greater than zero"),
        }
    }
}

impl std::error::Error for KernelError {}

struct Cell {
    mass: Mass,
}

struct Inert {
    mass: Mass,
}

/// The closed box.
pub struct World {
    next_cell: u64,
    next_inert: u64,
    cells: HashMap<u64, Cell>,
    inert: HashMap<u64, Inert>,
    house_burned: Mass,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            next_cell: 0,
            next_inert: 0,
            cells: HashMap::new(),
            inert: HashMap::new(),
            house_burned: Mass::ZERO,
        }
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

    pub fn cell_mass(&self, id: CellId) -> Option<Mass> {
        self.cells.get(&id.0).map(|c| c.mass)
    }

    pub fn inert_mass(&self, id: InertId) -> Option<Mass> {
        self.inert.get(&id.0).map(|i| i.mass)
    }

    /// Cash-in: new mass enters the box as a living cell.
    pub fn spawn_cell(&mut self, mass: Mass) -> Result<CellId, KernelError> {
        if mass.is_zero() {
            return Err(KernelError::ZeroAmount);
        }
        let id = CellId(self.next_cell);
        self.next_cell += 1;
        self.cells.insert(id.0, Cell { mass });
        Ok(id)
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
        let remaining = {
            let cell = self.cells.get_mut(&id.0).ok_or(KernelError::UnknownCell)?;
            cell.mass
                .checked_sub(amount)
                .ok_or(KernelError::InsufficientMass)?
        };
        if remaining.is_zero() {
            self.cells.remove(&id.0);
        } else {
            self.cells.get_mut(&id.0).unwrap().mass = remaining;
        }
        let inert_id = InertId(self.next_inert);
        self.next_inert += 1;
        self.inert.insert(inert_id.0, Inert { mass: amount });
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_adds_to_total_mass() {
        let mut w = World::new();
        assert_eq!(w.total_mass(), Mass::ZERO);
        assert_eq!(w.house_burned(), Mass::ZERO);
        w.spawn_cell(Mass::new(100)).unwrap();
        assert_eq!(w.total_mass(), Mass::new(100));
        assert_eq!(w.house_burned(), Mass::ZERO);
    }

    #[test]
    fn dump_and_absorb_conserve_mass() {
        let mut w = World::new();
        let a = w.spawn_cell(Mass::new(50)).unwrap();
        let b = w.spawn_cell(Mass::new(30)).unwrap();
        let before = w.total_mass();
        let dump = w.dump_matter(a, Mass::new(20)).unwrap();
        assert_eq!(w.total_mass(), before);
        assert_eq!(w.cell_mass(a), Some(Mass::new(30)));
        assert_eq!(w.inert_mass(dump), Some(Mass::new(20)));
        w.absorb_matter(b, dump, Mass::new(20)).unwrap();
        assert_eq!(w.total_mass(), before);
        assert_eq!(w.cell_mass(b), Some(Mass::new(50)));
        assert_eq!(w.inert_mass(dump), None);
        assert_eq!(w.house_burned(), Mass::ZERO);
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
        // Internal transfers still conserve what remains.
        let before = w.total_mass();
        w.absorb_matter(c, dump, Mass::new(10)).unwrap();
        assert_eq!(w.total_mass(), before);
        assert_eq!(w.house_burned(), Mass::new(15));
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
    }
}
