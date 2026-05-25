//! Voxel line-of-sight (FR-CIV-TACTICS-020).

use civ_voxel::{MaterialId, VoxelWorld, WorldCoord};

fn material_is_solid(material: MaterialId) -> bool {
    material.0 != 0
}

/// Integer grid step count along a 3D segment (Chebyshev / max-axis).
fn segment_steps(from: WorldCoord, to: WorldCoord) -> i64 {
    let dx = (to.x - from.x).abs();
    let dy = (to.y - from.y).abs();
    let dz = (to.z - from.z).abs();
    dx.max(dy).max(dz)
}

/// Returns true when no solid voxel blocks the segment between `from` and `to`.
///
/// Endpoints are not treated as blockers (only strictly between).
pub fn line_of_sight(world: &VoxelWorld<MaterialId>, from: WorldCoord, to: WorldCoord) -> bool {
    let steps = segment_steps(from, to);
    if steps == 0 {
        return true;
    }
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let dz = to.z - from.z;
    for step in 1..steps {
        let pos = WorldCoord {
            x: from.x + (dx * step) / steps,
            y: from.y + (dy * step) / steps,
            z: from.z + (dz * step) / steps,
        };
        if material_is_solid(world.read(pos)) {
            return false;
        }
    }
    true
}
