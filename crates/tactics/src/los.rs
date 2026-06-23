//! Voxel line-of-sight (FR-CIV-TACTICS-020).
//!
//! Implements 3D Bresenham integer raycasting.  The algorithm drives the ray
//! along the dominant axis and uses error accumulators (à la the 2D Bresenham
//! raster scan) to step the two minor axes.  All arithmetic is integer-only and
//! branch-free per voxel, matching the performance properties expected for
//! per-soldier LOS checks at tactical update frequency.
//!
//! ## Endpoint semantics
//! Neither `from` nor `to` is tested for solidity — the caller owns those cells
//! (typically the shooter's feet and the target's centre).

use civ_voxel::{MaterialId, VoxelWorld, WorldCoord};

fn material_is_solid(material: MaterialId) -> bool {
    material.0 != 0
}

/// Returns `true` when no solid voxel lies strictly between `from` and `to`.
///
/// Uses 3D Bresenham traversal over the dominant axis so every intermediate
/// grid cell is visited exactly once with no floating-point rounding.
///
/// Endpoints are **not** tested (a unit may stand inside a wall without that
/// blocking its own shots).
pub fn line_of_sight(world: &VoxelWorld<MaterialId>, from: WorldCoord, to: WorldCoord) -> bool {
    // Signed deltas.
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let dz = to.z - from.z;

    // Absolute step magnitudes.
    let ax = dx.abs();
    let ay = dy.abs();
    let az = dz.abs();

    // Unit step directions (+1 / -1 / 0).
    let sx = dx.signum();
    let sy = dy.signum();
    let sz = dz.signum();

    // Identify the dominant axis and the two minor axes.  The dominant axis
    // drives the outer loop; the minor-axis error accumulators are initialised
    // to half the dominant-axis length to produce the classic Bresenham
    // midpoint rounding.
    let (dom, minor_a_abs, minor_b_abs) = if ax >= ay && ax >= az {
        (ax, ay, az)
    } else if ay >= ax && ay >= az {
        (ay, ax, az)
    } else {
        (az, ax, ay)
    };

    if dom == 0 {
        // `from` and `to` are the same cell — trivially clear.
        return true;
    }

    // Error accumulators, initialised to half-dominant for symmetric rounding.
    let mut err_a = 2 * minor_a_abs - dom;
    let mut err_b = 2 * minor_b_abs - dom;

    // Current position (mutable copy of `from`).
    let mut x = from.x;
    let mut y = from.y;
    let mut z = from.z;

    // We visit `dom` steps; the final step would land exactly on `to`, which
    // we skip (endpoint-exclusion contract).
    for _ in 0..dom {
        // Advance dominant axis.
        if ax >= ay && ax >= az {
            x += sx;
            if err_a > 0 {
                y += sy;
                err_a -= 2 * dom;
            }
            if err_b > 0 {
                z += sz;
                err_b -= 2 * dom;
            }
            err_a += 2 * minor_a_abs;
            err_b += 2 * minor_b_abs;
        } else if ay >= ax && ay >= az {
            y += sy;
            if err_a > 0 {
                x += sx;
                err_a -= 2 * dom;
            }
            if err_b > 0 {
                z += sz;
                err_b -= 2 * dom;
            }
            err_a += 2 * minor_a_abs;
            err_b += 2 * minor_b_abs;
        } else {
            z += sz;
            if err_a > 0 {
                x += sx;
                err_a -= 2 * dom;
            }
            if err_b > 0 {
                y += sy;
                err_b -= 2 * dom;
            }
            err_a += 2 * minor_a_abs;
            err_b += 2 * minor_b_abs;
        }

        // Skip the final position (== `to`) — endpoint exclusion.
        if x == to.x && y == to.y && z == to.z {
            break;
        }

        if material_is_solid(world.read(WorldCoord { x, y, z })) {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Convenience constructor for a `VoxelWorld<MaterialId>` with unit scale.
    fn empty_world() -> VoxelWorld<MaterialId> {
        VoxelWorld::new(1)
    }

    fn wc(x: i64, y: i64, z: i64) -> WorldCoord {
        WorldCoord { x, y, z }
    }

    // -------------------------------------------------------------------------
    // FR-CIV-TACTICS-020 basic contract
    // -------------------------------------------------------------------------

    /// Clear LOS across empty space along the X axis.
    #[test]
    fn los_clear_along_x_axis() {
        let world = empty_world();
        assert!(line_of_sight(&world, wc(0, 0, 0), wc(10, 0, 0)));
    }

    /// Clear LOS across empty space along the Y axis.
    #[test]
    fn los_clear_along_y_axis() {
        let world = empty_world();
        assert!(line_of_sight(&world, wc(0, 0, 0), wc(0, 10, 0)));
    }

    /// Clear LOS across empty space along the Z axis.
    #[test]
    fn los_clear_along_z_axis() {
        let world = empty_world();
        assert!(line_of_sight(&world, wc(0, 0, 0), wc(0, 0, 10)));
    }

    /// Diagonal ray in the XY plane with no obstacles should be clear.
    #[test]
    fn los_clear_diagonal_xy() {
        let world = empty_world();
        assert!(line_of_sight(&world, wc(0, 0, 0), wc(8, 8, 0)));
    }

    /// Full 3D diagonal with no obstacles should be clear.
    #[test]
    fn los_clear_diagonal_3d() {
        let world = empty_world();
        assert!(line_of_sight(&world, wc(0, 0, 0), wc(5, 5, 5)));
    }

    // -------------------------------------------------------------------------
    // FR-CIV-TACTICS-020 wall blocking
    // -------------------------------------------------------------------------

    /// A wall of solid voxels filling all intermediate cells blocks LOS.
    #[test]
    fn los_blocked_by_wall_along_x() {
        let mut world = empty_world();
        let from = wc(0, 0, 0);
        let to = wc(8, 0, 0);
        // Fill everything between the endpoints.
        for x in 1..8 {
            world.write(wc(x, 0, 0), MaterialId(1));
        }
        assert!(!line_of_sight(&world, from, to));
    }

    /// A single solid voxel in the middle of the ray blocks LOS.
    #[test]
    fn los_blocked_by_single_voxel_midpoint() {
        let mut world = empty_world();
        let from = wc(0, 0, 0);
        let to = wc(10, 0, 0);
        world.write(wc(5, 0, 0), MaterialId(1));
        assert!(!line_of_sight(&world, from, to));
    }

    /// A blocker placed just before the target endpoint blocks LOS.
    #[test]
    fn los_blocked_near_target() {
        let mut world = empty_world();
        let from = wc(0, 0, 0);
        let to = wc(6, 0, 0);
        world.write(wc(5, 0, 0), MaterialId(1)); // one step before `to`
        assert!(!line_of_sight(&world, from, to));
    }

    /// A blocker placed just after the source endpoint blocks LOS.
    #[test]
    fn los_blocked_near_source() {
        let mut world = empty_world();
        let from = wc(0, 0, 0);
        let to = wc(6, 0, 0);
        world.write(wc(1, 0, 0), MaterialId(1)); // one step after `from`
        assert!(!line_of_sight(&world, from, to));
    }

    /// Removing the blocker restores LOS (regression: world mutation check).
    #[test]
    fn los_clear_after_blocker_removed() {
        let mut world = empty_world();
        let from = wc(0, 0, 0);
        let to = wc(8, 0, 0);
        world.write(wc(4, 0, 0), MaterialId(1));
        assert!(!line_of_sight(&world, from, to));
        world.write(wc(4, 0, 0), MaterialId(0));
        assert!(line_of_sight(&world, from, to));
    }

    // -------------------------------------------------------------------------
    // FR-CIV-TACTICS-020 endpoint exclusion
    // -------------------------------------------------------------------------

    /// A solid voxel at `from` does **not** block LOS (endpoint exclusion).
    #[test]
    fn los_solid_at_source_does_not_block() {
        let mut world = empty_world();
        let from = wc(0, 0, 0);
        let to = wc(5, 0, 0);
        world.write(from, MaterialId(1)); // source is solid — must be ignored
        assert!(line_of_sight(&world, from, to));
    }

    /// A solid voxel at `to` does **not** block LOS (endpoint exclusion).
    #[test]
    fn los_solid_at_target_does_not_block() {
        let mut world = empty_world();
        let from = wc(0, 0, 0);
        let to = wc(5, 0, 0);
        world.write(to, MaterialId(1)); // target is solid — must be ignored
        assert!(line_of_sight(&world, from, to));
    }

    // -------------------------------------------------------------------------
    // Edge cases
    // -------------------------------------------------------------------------

    /// Same position: trivially clear (zero-length segment).
    #[test]
    fn los_same_position_is_clear() {
        let world = empty_world();
        let pos = wc(3, 3, 3);
        assert!(line_of_sight(&world, pos, pos));
    }

    /// Adjacent positions with no voxel between them: clear.
    #[test]
    fn los_adjacent_clear() {
        let world = empty_world();
        assert!(line_of_sight(&world, wc(0, 0, 0), wc(1, 0, 0)));
        assert!(line_of_sight(&world, wc(0, 0, 0), wc(0, 1, 0)));
        assert!(line_of_sight(&world, wc(0, 0, 0), wc(0, 0, 1)));
    }

    /// Negative-direction ray: clear across empty space.
    #[test]
    fn los_negative_direction_clear() {
        let world = empty_world();
        assert!(line_of_sight(&world, wc(10, 10, 10), wc(0, 0, 0)));
    }

    /// Negative-direction ray blocked by a voxel.
    #[test]
    fn los_negative_direction_blocked() {
        let mut world = empty_world();
        world.write(wc(5, 5, 5), MaterialId(1));
        assert!(!line_of_sight(&world, wc(10, 10, 10), wc(0, 0, 0)));
    }

    /// Ray along Y-dominant axis with minor X deviation — clear.
    #[test]
    fn los_y_dominant_clear() {
        let world = empty_world();
        // dy=10 > dx=2, dz=0 — Y is dominant
        assert!(line_of_sight(&world, wc(0, 0, 0), wc(2, 10, 0)));
    }

    /// Ray along Z-dominant axis blocked mid-way.
    #[test]
    fn los_z_dominant_blocked() {
        let mut world = empty_world();
        world.write(wc(0, 0, 5), MaterialId(1));
        assert!(!line_of_sight(&world, wc(0, 0, 0), wc(0, 0, 10)));
    }

    /// Symmetric: LOS(A→B) == LOS(B→A).
    #[test]
    fn los_is_symmetric() {
        let mut world = empty_world();
        let a = wc(0, 0, 0);
        let b = wc(8, 0, 0);

        // Empty world — both directions clear.
        assert_eq!(line_of_sight(&world, a, b), line_of_sight(&world, b, a));

        // Add a blocker.
        world.write(wc(4, 0, 0), MaterialId(1));
        assert_eq!(line_of_sight(&world, a, b), line_of_sight(&world, b, a));
    }
}
