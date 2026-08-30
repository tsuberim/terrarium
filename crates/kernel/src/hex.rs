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

/// Absolute direction from body-facing + relative step (0 = forward, 1 = right, …).
pub fn abs_dir(facing: u8, relative: u8) -> u8 {
    (facing + relative % 6) % 6
}

/// Nearest hex direction toward offset `(dq, dr)` from origin.
pub fn direction_toward(dq: i32, dr: i32) -> Option<u8> {
    if dq == 0 && dr == 0 {
        return None;
    }
    let qf = dq as f64;
    let rf = dr as f64;
    let sf = -qf - rf;
    let scale = qf.abs().max(rf.abs()).max(sf.abs());
    let qf = qf / scale;
    let rf = rf / scale;
    let sf = sf / scale;
    let mut rq = qf.round() as i32;
    let mut rr = rf.round() as i32;
    let mut rs = sf.round() as i32;
    let q_diff = (rq as f64 - qf).abs();
    let r_diff = (rr as f64 - rf).abs();
    let s_diff = (rs as f64 - sf).abs();
    if q_diff > r_diff && q_diff > s_diff {
        rq = -rr - rs;
    } else if r_diff > s_diff {
        rr = -rq - rs;
    } else {
        rs = -rq - rr;
    }
    dir_of_offset(rq, rr)
}

/// Signed bearing from `facing` toward `(dq, dr)`: 0 = ahead, ±1 = 60° off, etc.
pub fn relative_bearing(facing: u8, dq: i32, dr: i32) -> Option<i32> {
    let target = direction_toward(dq, dr)?;
    let diff = (target as i32 - facing as i32).rem_euclid(6);
    Some(if diff > 3 { diff - 6 } else { diff })
}

/// True when `(dq, dr)` is within frontal arc ±`half_arc` hex steps (1 = 120° cone).
pub fn in_fov(facing: u8, dq: i32, dr: i32, half_arc: i32) -> bool {
    if dq == 0 && dr == 0 {
        return true;
    }
    relative_bearing(facing, dq, dr)
        .map(|b| b.abs() <= half_arc)
        .unwrap_or(false)
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

    #[test]
    fn fov_cone_ahead_only_at_distance() {
        assert!(in_fov(0, 1, 0, 1)); // E ahead
        assert!(in_fov(0, 2, 0, 1)); // E at distance 2
        assert!(!in_fov(0, -1, 0, 1)); // W behind
        assert!(in_fov(0, 1, -1, 1)); // NE (bearing +1)
        assert!(!in_fov(0, -1, 1, 1)); // SW (bearing ±2)
    }
}
