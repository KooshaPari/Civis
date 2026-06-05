#![cfg(feature = "voxel")]

use civ_bevy_ref::voxel_sim::world_dims_for;

#[test]
fn world_size_selection_maps_to_increasing_dimensions() {
    let small = world_dims_for(0);
    let large = world_dims_for(2);

    assert!(large[0] > small[0], "large width should exceed small width");
    assert!(large[1] >= small[1], "large height should not shrink below small height");
    assert!(large[2] > small[2], "large depth should exceed small depth");
}
