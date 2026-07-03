//! External integration tests for crates/voxel coverage gaps.
//!
//! Covers: contains_world_coord (zero-span, inside, outside) from boundary.rs.
use civ_voxel::{
    boundary::contains_world_coord,
    Bounds3, WorldCoord,
};

fn wc(x: i64, y: i64, z: i64) -> WorldCoord {
    WorldCoord { x, y, z }
}

fn bounds(min: [i32; 3], max: [i32; 3]) -> Bounds3 {
    Bounds3 { min, max }
}

#[test]
fn contains_world_coord_inside() {
    // voxel_span=1: cell = coord itself; bounds [0,4) in each axis
    let b = bounds([0, 0, 0], [4, 4, 4]);
    assert!(contains_world_coord(b, 1, wc(0, 0, 0)));
    assert!(contains_world_coord(b, 1, wc(3, 3, 3)));
    assert!(!contains_world_coord(b, 1, wc(4, 0, 0)), "max is exclusive");
    assert!(!contains_world_coord(b, 1, wc(-1, 0, 0)), "below min");
}

#[test]
fn contains_world_coord_zero_span_always_false() {
    let b = bounds([0, 0, 0], [4, 4, 4]);
    // voxel_span=0 is a degenerate case the fn guards against
    assert!(!contains_world_coord(b, 0, wc(1, 1, 1)));
}

#[test]
fn contains_world_coord_with_larger_span() {
    // voxel_span=10: coord 25 maps to cell 2 (25 / 10 = 2), inside [0,4)
    let b = bounds([0, 0, 0], [4, 4, 4]);
    assert!(contains_world_coord(b, 10, wc(25, 0, 0)));
    // coord 40 maps to cell 4 — outside [0,4)
    assert!(!contains_world_coord(b, 10, wc(40, 0, 0)));
}