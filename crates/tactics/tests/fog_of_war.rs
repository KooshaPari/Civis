//! Fog-of-war coverage tests for the tactical visibility grid.
//!
//! These tests exercise the public `FogOfWar` API from the perspective of an
//! external consumer so the visibility contract stays stable.

use civ_tactics::{grid_to_world_coord, FogOfWar, MilitaryUnitSample};
use civ_voxel::{MaterialId, VoxelWorld, WorldCoord};

fn empty_world() -> VoxelWorld<MaterialId> {
    VoxelWorld::new(1)
}

fn unit(unit_id: u64, faction_id: u32, grid_x: i32, grid_y: i32) -> MilitaryUnitSample {
    MilitaryUnitSample {
        unit_id,
        faction_id,
        grid_x,
        grid_y,
    }
}

fn reveal_column(world: &mut VoxelWorld<MaterialId>, grid_x: i32) {
    let wall_wc = grid_to_world_coord(grid_x, 0);
    for dy in -5_i64..=5 {
        world.write(
            WorldCoord {
                x: wall_wc.x,
                y: dy,
                z: wall_wc.z,
            },
            MaterialId(1),
        );
    }
}

#[test]
fn fog_cell_starts_hidden() {
    let fog = FogOfWar::new(16, None);

    assert_eq!(fog.grid_size(), 16);
    assert_eq!(fog.vision_radius(), 8);
    assert!(!fog.is_visible(0, (0, 0)));
    assert!(!fog.is_visible(0, (7, 7)));
}

#[test]
fn reveal_cell_makes_it_visible() {
    let mut fog = FogOfWar::new(16, Some(4));
    let world = empty_world();
    let units = [unit(1, 0, 3, 4)];

    fog.update(&units, &world);

    assert!(fog.is_visible(0, (3, 4)));
}

#[test]
fn visibility_propagates_through_line_of_sight() {
    let mut fog = FogOfWar::new(16, Some(6));
    let mut world = empty_world();
    reveal_column(&mut world, 3);

    let units = [unit(1, 0, 2, 0)];
    fog.update(&units, &world);

    assert!(fog.is_visible(0, (1, 0)));
    assert!(!fog.is_visible(0, (4, 0)));
}

#[test]
fn moving_unit_reveals_new_cells_and_hides_old_ones_outside_los_range() {
    let mut fog = FogOfWar::new(32, Some(3));
    let world = empty_world();
    let mut units = [unit(1, 0, 2, 2)];

    fog.update(&units, &world);
    assert!(fog.is_visible(0, (2, 2)));

    units[0].grid_x = 20;
    units[0].grid_y = 20;
    fog.update(&units, &world);

    assert!(fog.is_visible(0, (20, 20)));
    assert!(!fog.is_visible(0, (2, 2)));
}

#[test]
fn fog_resets_on_new_game() {
    let mut fog = FogOfWar::new(16, Some(4));
    let world = empty_world();
    let units = [unit(1, 0, 3, 3)];

    fog.update(&units, &world);
    assert!(fog.is_visible(0, (3, 3)));

    fog.update(&[], &world);
    assert!(!fog.is_visible(0, (3, 3)));
    assert!(!fog.is_visible(0, (4, 3)));
}
