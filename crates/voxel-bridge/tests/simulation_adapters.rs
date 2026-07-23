use civ_voxel::WorldCoord;
use civ_voxel_bridge::{boundary, fluid_ca, reactions};

#[test]
fn simulation_adapters_use_kernel_contracts() {
    let bounds = boundary::Bounds3::from_origin_size([0, 0, 0], [4, 4, 4]);
    assert!(boundary::contains_world_coord(
        bounds,
        1,
        WorldCoord { x: 3, y: 3, z: 3 }
    ));
    assert!(!boundary::contains_world_coord(
        bounds,
        1,
        WorldCoord { x: 4, y: 0, z: 0 }
    ));

    let rule = reactions::reaction_for(civ_voxel::LAVA, civ_voxel::WATER);
    assert!(rule.is_some());

    let grid = fluid_ca::CaGrid::new([2, 2, 2]);
    assert_eq!(grid.cells.len(), 8);
}
