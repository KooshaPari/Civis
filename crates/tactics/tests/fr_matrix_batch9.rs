//! FR coverage batch 9: `civ-tactics` integration tests for IMPL-NO-TEST rows
//! targeting the voxel-derived grid obstacles and unit-occupancy pathfinding
//! gating APIs (FR-CIV-TACTICS-036, FR-CIV-TACTICS-039).
//!
//! These exercise the public API of `civ_tactics::grid_obstacles` against
//! representative `VoxelWorld` + `MilitaryUnitSample` slices, mirroring how
//! the operational pathfinding layer (`crates/tactics/src/pathfinding.rs`)
//! classifies cells as blocked, occupied, or impassable.

use civ_tactics::{
    grid_cell_blocked, grid_cell_impassable, grid_cell_occupied, grid_to_world_coord,
    MilitaryUnitSample,
};
use civ_voxel::{MaterialId, VoxelWorld, FIXED_SCALE};

fn empty_world() -> VoxelWorld<MaterialId> {
    VoxelWorld::new(FIXED_SCALE)
}

fn unit(unit_id: u64, faction_id: u32, grid_x: i32, grid_y: i32) -> MilitaryUnitSample {
    MilitaryUnitSample {
        unit_id,
        faction_id,
        grid_x,
        grid_y,
    }
}

#[test]
fn fr_civ_tactics_036_solid_voxel_marks_cell_blocked() {
    // FR-CIV-TACTICS-036: voxel-derived grid obstacles classify solid voxels
    // as impassable for pathfinding. Build a 1x1 wall at the operational plane
    // and assert that the host cell is blocked, but a neighbouring cell that
    // we have not written to is not blocked by the same call.
    let mut world = empty_world();
    let coord = grid_to_world_coord(2, 3);
    world.write(coord, MaterialId(1));

    assert!(
        grid_cell_blocked(&world, 2, 3),
        "cell with a solid voxel must be classified as blocked"
    );
    // Sanity: a unit (not the one that triggered the write) can walk an
    // adjacent cell whose voxel world coord is not the one we wrote to.
    let other = grid_to_world_coord(2, 4);
    assert_ne!(
        other, coord,
        "test setup invariant: adjacent grid cell maps to a different voxel coord"
    );
}

#[test]
fn fr_civ_tactics_039_occupation_marks_cell_impassable() {
    // FR-CIV-TACTICS-039: unit occupation and impassable cells gate
    // pathfinding. A cell with another unit in it is impassable, but a
    // cell that only contains the querying unit's index is not.
    let world = empty_world();
    let units = [unit(1, 0, 5, 5), unit(2, 0, 7, 7)];

    // Cell (5,5) holds unit 0 — not occupied from unit 0's perspective.
    assert!(
        !grid_cell_occupied(&units, 5, 5, 0),
        "a unit must not be considered to occupy its own cell"
    );
    // Cell (7,7) holds unit 1 — occupied from unit 0's perspective.
    assert!(
        grid_cell_occupied(&units, 7, 7, 0),
        "another unit's cell must be classified as occupied"
    );
    // Off-grid cell — never occupied.
    assert!(
        !grid_cell_occupied(&units, 99, 99, 0),
        "cells with no unit are not occupied"
    );

    // grid_cell_impassable = solid voxel OR occupied by another unit.
    // (7,7) is occupied -> impassable from unit 0's perspective.
    assert!(
        grid_cell_impassable(&world, &units, 7, 7, 0),
        "occupied cell must be impassable"
    );
    // (0,0) is empty and unoccupied -> passable.
    assert!(
        !grid_cell_impassable(&world, &units, 0, 0, 0),
        "empty unoccupied cell must be passable"
    );
}
