//! Boundary helpers for finite voxel domains.
//!
//! The simulation code in this crate operates on bounded regions even though the
//! underlying `VoxelWorld` storage is sparse. This module defines the finite box
//! and provides helpers to seed and enforce solid edge cells.

use crate::{MaterialId, VoxelWorld, WorldCoord};

/// The six axis-aligned faces of a bounded voxel region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryFace {
    /// Minimum `x` face.
    NegX,
    /// Maximum `x` face (exclusive upper bound).
    PosX,
    /// Minimum `y` face.
    NegY,
    /// Maximum `y` face.
    PosY,
    /// Minimum `z` face.
    NegZ,
    /// Maximum `z` face.
    PosZ,
}

impl BoundaryFace {
    /// Index helper for fixed-size arrays.
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::NegX => 0,
            Self::PosX => 1,
            Self::NegY => 2,
            Self::PosY => 3,
            Self::NegZ => 4,
            Self::PosZ => 5,
        }
    }
}

/// Per-face behavior for boundary interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryMode {
    /// Delete touching fluids/gases and clamp heat to ambient.
    Vacuum,
    /// Inject a material from this face.
    Inflow {
        /// Material to seed into edge cells.
        material: MaterialId,
        /// Seed chance in 0-255 where 255 is always.
        rate: u8,
        /// Temperature for seeded cells.
        temp: i16,
    },
    /// Keep cells in this face unchanged.
    Closed,
}

/// Boundary controller used by CA passes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundaryConfig {
    /// Per-face behavior.
    pub faces: [BoundaryMode; 6],
    /// Ambient temperature used for ghost neighbor interaction.
    pub ambient_temp: i16,
}

impl BoundaryConfig {
    /// Returns the default boundary configuration (all closed, 20°C ambient).
    #[must_use]
    pub const fn closed() -> Self {
        Self {
            faces: [BoundaryMode::Closed; 6],
            ambient_temp: 20,
        }
    }
}

/// Inclusive-min, exclusive-max bounds in voxel coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bounds3 {
    /// Minimum voxel coordinate included in the region.
    pub min: [i32; 3],
    /// Exclusive maximum voxel coordinate.
    pub max: [i32; 3],
}

impl Bounds3 {
    /// Construct bounds from an origin and size.
    #[must_use]
    pub const fn from_origin_size(origin: [i32; 3], size: [i32; 3]) -> Self {
        Self {
            min: origin,
            max: [origin[0] + size[0], origin[1] + size[1], origin[2] + size[2]],
        }
    }

    /// Returns `true` when `cell` lies inside the bounded domain.
    #[must_use]
    pub const fn contains_cell(self, cell: [i32; 3]) -> bool {
        cell[0] >= self.min[0]
            && cell[0] < self.max[0]
            && cell[1] >= self.min[1]
            && cell[1] < self.max[1]
            && cell[2] >= self.min[2]
            && cell[2] < self.max[2]
    }

    /// Returns `true` when `cell` lies on any domain edge.
    #[must_use]
    pub const fn is_boundary_cell(self, cell: [i32; 3]) -> bool {
        self.contains_cell(cell)
            && (cell[0] == self.min[0]
                || cell[0] == self.max[0] - 1
                || cell[1] == self.min[1]
                || cell[1] == self.max[1] - 1
                || cell[2] == self.min[2]
                || cell[2] == self.max[2] - 1)
    }

    /// Returns `true` when `cell` is on the floor plane.
    #[must_use]
    pub const fn is_floor_cell(self, cell: [i32; 3]) -> bool {
        self.contains_cell(cell) && cell[1] == self.min[1]
    }
}

fn cell_to_world(voxel_span: i64, cell: [i32; 3]) -> WorldCoord {
    WorldCoord {
        x: i64::from(cell[0]) * voxel_span,
        y: i64::from(cell[1]) * voxel_span,
        z: i64::from(cell[2]) * voxel_span,
    }
}

/// Returns `true` when the coordinate is inside the bounded domain.
#[must_use]
pub fn contains_world_coord(bounds: Bounds3, voxel_span: i64, coord: WorldCoord) -> bool {
    if voxel_span == 0 {
        return false;
    }
    let cell = [
        coord.x.div_euclid(voxel_span) as i32,
        coord.y.div_euclid(voxel_span) as i32,
        coord.z.div_euclid(voxel_span) as i32,
    ];
    bounds.contains_cell(cell)
}

/// Seed the solid domain edges and floor with `wall_material`.
pub fn seed_boundary_walls(
    world: &mut VoxelWorld<MaterialId>,
    voxel_span: i64,
    bounds: Bounds3,
    wall_material: MaterialId,
) {
    for x in bounds.min[0]..bounds.max[0] {
        for y in bounds.min[1]..bounds.max[1] {
            for z in bounds.min[2]..bounds.max[2] {
                let cell = [x, y, z];
                if bounds.is_boundary_cell(cell) || bounds.is_floor_cell(cell) {
                    world.write(cell_to_world(voxel_span, cell), wall_material);
                }
            }
        }
    }
}

/// Re-enforce the boundary walls after a simulation step.
pub fn enforce_boundary_walls(
    world: &mut VoxelWorld<MaterialId>,
    voxel_span: i64,
    bounds: Bounds3,
    wall_material: MaterialId,
) {
    seed_boundary_walls(world, voxel_span, bounds, wall_material);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::{BEDROCK, WATER};

    #[test]
    fn containment_and_edges_are_consistent() {
        let bounds = Bounds3::from_origin_size([0, 0, 0], [8, 4, 8]);
        assert!(bounds.contains_cell([0, 0, 0]));
        assert!(bounds.contains_cell([7, 3, 7]));
        assert!(!bounds.contains_cell([8, 3, 7]));
        assert!(bounds.is_boundary_cell([0, 2, 4]));
        assert!(bounds.is_floor_cell([3, 0, 3]));
        assert!(!bounds.is_floor_cell([3, 1, 3]));
    }

    #[test]
    fn boundary_seeding_writes_floor_and_walls() {
        let bounds = Bounds3::from_origin_size([0, 0, 0], [4, 4, 4]);
        let mut world = VoxelWorld::new(1);
        seed_boundary_walls(&mut world, 1, bounds, BEDROCK);
        assert_eq!(world.read(cell_to_world(1, [0, 0, 0])), BEDROCK);
        assert_eq!(world.read(cell_to_world(1, [3, 3, 3])), BEDROCK);
        assert_eq!(world.read(cell_to_world(1, [1, 1, 1])), MaterialId(0));
        assert!(contains_world_coord(bounds, 1, cell_to_world(1, [2, 2, 2])));
        assert!(!contains_world_coord(bounds, 1, cell_to_world(1, [4, 2, 2])));
        assert_ne!(world.read(cell_to_world(1, [1, 1, 1])), WATER);
    }
}
