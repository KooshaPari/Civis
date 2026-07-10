//! Footprint brush primitive for `sim.terraform_extent` (FR-CIV-GODTOOL brush).
//!
//! Stamps a circular (disk) footprint of material writes around a center.
//! Used by the server bridge and MCP `sim_terraform_extent` so agents do not
//! have to loop `sim.place_voxel` per cell.

use crate::{MaterialId, VoxelWorld, WorldCoord, FIXED_SCALE};

/// Brush shape for footprint stamping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrushShape {
    /// Filled disk in the XZ plane (default).
    Disk,
}

/// Parameters for a single footprint stamp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrushStamp {
    /// World-space center (fixed-point).
    pub center: WorldCoord,
    /// Radius in voxel cells (not fixed-point).
    pub radius_voxels: u8,
    /// Material written into each cell of the footprint.
    pub material: MaterialId,
    /// Brush shape.
    pub shape: BrushShape,
    /// Optional vertical band thickness in voxel cells (default 1).
    pub height_voxels: u8,
}

/// Result of applying a brush stamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrushReceipt {
    /// Number of voxel cells written.
    pub writes: u32,
}

/// Stamp a footprint into `world`, returning the write count.
///
/// Disk stamps write `height_voxels` layers centered on `center.y`.
pub fn stamp_footprint(world: &mut VoxelWorld<MaterialId>, stamp: &BrushStamp) -> BrushReceipt {
    let r = i64::from(stamp.radius_voxels.max(1));
    let r2 = r * r;
    let h = i64::from(stamp.height_voxels.max(1));
    let cx = stamp.center.x;
    let cy = stamp.center.y;
    let cz = stamp.center.z;
    let mut writes: u32 = 0;

    match stamp.shape {
        BrushShape::Disk => {
            for dz in -r..=r {
                for dx in -r..=r {
                    if dx * dx + dz * dz > r2 {
                        continue;
                    }
                    for dy in 0..h {
                        let y_off = dy - h / 2;
                        world.write(
                            WorldCoord {
                                x: cx + dx * FIXED_SCALE,
                                y: cy + y_off * FIXED_SCALE,
                                z: cz + dz * FIXED_SCALE,
                            },
                            stamp.material,
                        );
                        writes = writes.saturating_add(1);
                    }
                }
            }
        }
    }

    BrushReceipt { writes }
}

/// Map a terraform op name to a default material for best-effort stamping.
#[must_use]
pub fn default_material_for_op(op: &str) -> MaterialId {
    match op {
        "lower" | "dig_ocean" | "erase" => crate::AIR,
        "drop_biome" => crate::SAND,
        "pour_liquid" | "flood" => crate::WATER,
        "seed_snow" => crate::SNOW,
        "seed_ore" => crate::ORE,
        "seed_forest" => crate::PLANT,
        _ => crate::STONE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VoxelWorld;

    #[test]
    fn stamp_disk_writes_expected_cells() {
        let mut world = VoxelWorld::<MaterialId>::new(FIXED_SCALE);
        let receipt = stamp_footprint(
            &mut world,
            &BrushStamp {
                center: WorldCoord { x: 0, y: 0, z: 0 },
                radius_voxels: 1,
                material: crate::STONE,
                shape: BrushShape::Disk,
                height_voxels: 1,
            },
        );
        // radius 1 disk = 5 cells (center + 4 cardinals)
        assert_eq!(receipt.writes, 5);
        assert_eq!(world.read(WorldCoord { x: 0, y: 0, z: 0 }), crate::STONE);
    }
}
