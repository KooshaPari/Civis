//! Voxel-derived grid obstacles for operational pathfinding (FR-CIV-TACTICS-036).

use crate::war_bridge::grid_to_world_coord;
use civ_voxel::{MaterialId, VoxelWorld};

/// Returns true when the grid cell has a solid voxel at the operational plane.
#[must_use]
pub fn grid_cell_blocked(world: &VoxelWorld<MaterialId>, grid_x: i32, grid_y: i32) -> bool {
    let coord = grid_to_world_coord(grid_x, grid_y);
    world.read(coord).0 != 0
}
