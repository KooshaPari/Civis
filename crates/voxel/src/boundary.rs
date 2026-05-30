//! World-boundary helpers for bounded voxel simulations.

use crate::MaterialId;

/// Dense-grid bounds for a voxel world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bounds {
    /// World dimensions in `[x, y, z]` order.
    pub dims: [usize; 3],
}

impl Bounds {
    /// Returns the flattened index for an in-bounds coordinate.
    #[must_use]
    pub fn idx(self, x: usize, y: usize, z: usize) -> usize {
        x + y * self.dims[0] + z * self.dims[0] * self.dims[1]
    }

    /// Returns `true` when the coordinate lies within the world dimensions.
    #[must_use]
    pub fn in_bounds(self, x: usize, y: usize, z: usize) -> bool {
        x < self.dims[0] && y < self.dims[1] && z < self.dims[2]
    }

    /// Returns `true` when the coordinate is on any outer face of the box.
    #[must_use]
    pub fn is_edge(self, x: usize, y: usize, z: usize) -> bool {
        self.in_bounds(x, y, z)
            && (x == 0
                || y == 0
                || z == 0
                || x + 1 == self.dims[0]
                || y + 1 == self.dims[1]
                || z + 1 == self.dims[2])
    }
}

/// Sets all edge cells of a dense voxel grid to `wall`.
pub fn seal_walls(cells: &mut [MaterialId], dims: [usize; 3], wall: MaterialId) {
    let bounds = Bounds { dims };
    for z in 0..dims[2] {
        for y in 0..dims[1] {
            for x in 0..dims[0] {
                if bounds.is_edge(x, y, z) {
                    cells[bounds.idx(x, y, z)] = wall;
                }
            }
        }
    }
}

/// Converts a signed coordinate into an in-bounds grid coordinate.
#[must_use]
pub fn clamp_coord(dims: [usize; 3], x: i64, y: i64, z: i64) -> Option<(usize, usize, usize)> {
    let ux = usize::try_from(x).ok()?;
    let uy = usize::try_from(y).ok()?;
    let uz = usize::try_from(z).ok()?;
    let bounds = Bounds { dims };
    bounds.in_bounds(ux, uy, uz).then_some((ux, uy, uz))
}

/// Returns `true` when a neighbor lookup falls outside the world bounds.
#[must_use]
pub fn neighbor_is_wall(dims: [usize; 3], x: i64, y: i64, z: i64) -> bool {
    clamp_coord(dims, x, y, z).is_none()
}

/// Verifies that every edge cell in the grid equals `wall`.
#[must_use]
pub fn assert_sealed(cells: &[MaterialId], dims: [usize; 3], wall: MaterialId) -> bool {
    let bounds = Bounds { dims };
    for z in 0..dims[2] {
        for y in 0..dims[1] {
            for x in 0..dims[0] {
                if bounds.is_edge(x, y, z) && cells[bounds.idx(x, y, z)] != wall {
                    return false;
                }
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::{AIR, BEDROCK};

    #[test]
    fn seal_then_assert() {
        let dims = [4, 4, 4];
        let mut cells = vec![AIR; dims[0] * dims[1] * dims[2]];
        let interior_idx = Bounds { dims }.idx(1, 1, 1);
        seal_walls(&mut cells, dims, BEDROCK);
        assert!(assert_sealed(&cells, dims, BEDROCK));
        assert_eq!(cells[interior_idx], AIR);
    }

    #[test]
    fn out_of_bounds_is_wall() {
        let dims = [4, 3, 2];
        assert!(neighbor_is_wall(dims, -1, 1, 1));
        assert!(neighbor_is_wall(dims, 4, 1, 1));
        assert!(!neighbor_is_wall(dims, 1, 1, 1));
    }

    #[test]
    fn clamp() {
        let dims = [4, 3, 2];
        assert_eq!(clamp_coord(dims, 1, 2, 1), Some((1, 2, 1)));
        assert_eq!(clamp_coord(dims, -1, 0, 0), None);
        assert_eq!(clamp_coord(dims, 4, 0, 0), None);
    }

    #[test]
    fn edge_detection() {
        let bounds = Bounds { dims: [4, 4, 4] };
        assert!(bounds.is_edge(0, 1, 1));
        assert!(bounds.is_edge(3, 1, 1));
        assert!(bounds.is_edge(1, 0, 1));
        assert!(bounds.is_edge(1, 3, 1));
        assert!(bounds.is_edge(1, 1, 0));
        assert!(!bounds.is_edge(1, 1, 1));
    }
}
