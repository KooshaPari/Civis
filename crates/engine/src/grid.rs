//! FR-CORE-009 — Hex grid axial coordinate types.
//!
//! Provides [`PositionAxial`] using hexx 0.21.x-style axial coordinates (q, r)
//! and cube-to-axial / axial-to-cube roundtrip conversions.
//!
//! Cube coordinates: (x, y, z) where x + y + z = 0.
//! Axial coordinates: (q, r) where q = x, r = z (y derived as -q - r).

use serde::{Deserialize, Serialize};

/// Axial hex coordinate using hexx 0.21.x convention.
///
/// `q` is the column index, `r` is the row index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PositionAxial {
    /// Column coordinate (maps to cube x).
    pub q: i32,
    /// Row coordinate (maps to cube z).
    pub r: i32,
}

/// Cube hex coordinate where x + y + z = 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PositionCube {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl PositionAxial {
    /// Create a new axial position.
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    /// Convert from cube coordinates to axial.
    /// Axial q = cube x, axial r = cube z.
    pub fn from_cube(cube: PositionCube) -> Self {
        debug_assert_eq!(cube.x + cube.y + cube.z, 0, "cube coords must sum to 0");
        Self {
            q: cube.x,
            r: cube.z,
        }
    }

    /// Convert to cube coordinates.
    /// Cube x = axial q, cube z = axial r, cube y = -q - r.
    pub fn to_cube(&self) -> PositionCube {
        PositionCube {
            x: self.q,
            y: -self.q - self.r,
            z: self.r,
        }
    }
}

impl PositionCube {
    /// Create a new cube position. Panics in debug if x + y + z != 0.
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        debug_assert_eq!(x + y + z, 0, "cube coords must sum to 0");
        Self { x, y, z }
    }

    /// Convert to axial coordinates.
    pub fn to_axial(&self) -> PositionAxial {
        PositionAxial::from_cube(*self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axial_roundtrip() {
        let axial = PositionAxial::new(3, -5);
        let cube = axial.to_cube();
        assert_eq!(cube.x + cube.y + cube.z, 0);
        let back = PositionAxial::from_cube(cube);
        assert_eq!(back, axial);
    }

    #[test]
    fn cube_roundtrip() {
        let cube = PositionCube::new(2, -3, 1);
        let axial = cube.to_axial();
        let back = axial.to_cube();
        assert_eq!(back, cube);
    }

    #[test]
    fn from_cube_sets_q_and_r() {
        let cube = PositionCube::new(4, -7, 3);
        let axial = PositionAxial::from_cube(cube);
        assert_eq!(axial.q, 4);
        assert_eq!(axial.r, 3);
    }

    #[test]
    fn to_cube_derives_y() {
        let axial = PositionAxial::new(5, -2);
        let cube = axial.to_cube();
        assert_eq!(cube.x, 5);
        assert_eq!(cube.y, -3);
        assert_eq!(cube.z, -2);
    }
}
