//! Voxel-derived grid obstacles for operational pathfinding (FR-CIV-TACTICS-036).

use crate::war_bridge::grid_to_world_coord;
use civ_voxel::{MaterialId, VoxelWorld};

/// Returns true when the grid cell has a solid voxel at the operational plane.
#[must_use]
pub fn grid_cell_blocked(world: &VoxelWorld<MaterialId>, grid_x: i32, grid_y: i32) -> bool {
    let coord = grid_to_world_coord(grid_x, grid_y);
    world.read(coord).0 != 0
}

/// Returns true when another unit occupies the grid cell (FR-CIV-TACTICS-039).
#[must_use]
pub fn grid_cell_occupied(
    units: &[crate::war_bridge::MilitaryUnitSample],
    grid_x: i32,
    grid_y: i32,
    skip_index: usize,
) -> bool {
    units
        .iter()
        .enumerate()
        .any(|(i, u)| i != skip_index && u.grid_x == grid_x && u.grid_y == grid_y)
}

/// Voxel solid or occupied by another unit — impassable for pathfinding (FR-CIV-TACTICS-039).
#[must_use]
pub fn grid_cell_impassable(
    world: &VoxelWorld<MaterialId>,
    units: &[crate::war_bridge::MilitaryUnitSample],
    grid_x: i32,
    grid_y: i32,
    skip_index: usize,
) -> bool {
    grid_cell_blocked(world, grid_x, grid_y)
        || grid_cell_occupied(units, grid_x, grid_y, skip_index)
}
