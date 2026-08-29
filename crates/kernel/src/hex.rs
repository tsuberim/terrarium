//! Axial hex coordinates (pointy-top). Stored as `x`/`y` in the sim — `x` = q, `y` = r.

use crate::abi::dir;

pub type Coord = (i32, i32);

/// Hex distance between two axial cells.
pub fn distance(q1: i32, r1: i32, q2: i32, r2: i32) -> i32 {
    let dq = (q1 - q2).abs();
    let dr = (r1 - r2).abs();
    let ds = (q1 + r1 - q2 - r2).abs();
    (dq + dr + ds) / 2
}

/// True when offset `(dq, dr)` from origin is within hex radius `r`.
pub fn in_range(dq: i32, dr: i32, r: i32) -> bool {
    distance(0, 0, dq, dr) <= r
}

/// Neighbor in direction 0–5 (E, NE, NW, W, SW, SE).
pub fn neighbor(q: i32, r: i32, dir: u8) -> Option<Coord> {
    Some(match dir {
        d if d == dir::E as u8 => (q + 1, r),
        d if d == dir::NE as u8 => (q + 1, r - 1),
        d if d == dir::NW as u8 => (q, r - 1),
        d if d == dir::W as u8 => (q - 1, r),
        d if d == dir::SW as u8 => (q - 1, r + 1),
        d if d == dir::SE as u8 => (q, r + 1),
        _ => return None,
    })
}

/// Direction 0–5 that steps from `(q,r)` toward offset `(dq,dr)` (must be a neighbor).
pub fn dir_of_offset(dq: i32, dr: i32) -> Option<u8> {
    for d in 0..6u8 {
        if neighbor(0, 0, d) == Some((dq, dr)) {
            return Some(d);
        }
    }
    None
}

/// Opposite hex direction (0–5).
pub fn opposite(dir: u8) -> u8 {
    (dir + 3) % 6
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neighbors_are_distance_one() {
        for d in 0..6u8 {
            let (q, r) = neighbor(0, 0, d).unwrap();
            assert_eq!(distance(0, 0, q, r), 1);
        }
    }

    #[test]
    fn range_is_disk_not_square() {
        assert!(in_range(1, 0, 1));
        assert!(in_range(1, -1, 1));
        assert!(!in_range(1, 1, 1)); // Chebyshev square would include this
    }

    #[test]
    fn opposite_round_trip() {
        for d in 0..6u8 {
            assert_eq!(opposite(opposite(d)), d);
        }
    }
}
