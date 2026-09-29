//!
//! Provenance: this test previously carried a `FR-UX-001` tag. That id is
//! defined in `docs/models/civ-sim/USER_SPEC.md` as an unrelated
//! requirement and is NOT implemented by this code. The only document
//! claiming otherwise was `docs/traceability/TRACEABILITY_MATRIX.md`,
//! which cited a nonexistent spec file. The assertions below are real and
//! are retained as a behavioral test of hex map draw-list culling and batching; the false id was removed
//! rather than rebound. See `docs/audits/id-provenance-corrections.md`.
//!

use civ_render::frame::TARGET_FPS;
use civ_render::hex_map::{HexCoord, HexMapRenderer, Tile, HEX_MAP_TARGET_FPS};

fn grid(n: i32) -> Vec<Tile> {
    let mut tiles = Vec::new();
    for q in -n..=n {
        for r in -n..=n {
            tiles.push(Tile {
                coord: HexCoord::new(q, r),
                terrain: u16::try_from((q + r).rem_euclid(4)).unwrap(),
                lod: 0,
            });
        }
    }
    tiles
}

/// The renderer targets 60 fps and its worst-case frame fits the budget.
#[test]
fn hex_map_60fps() {
    let mut renderer = HexMapRenderer::new(grid(40));
    renderer.set_view_radius(6);

    assert_eq!(renderer.target_fps(), TARGET_FPS);
    assert_eq!(HEX_MAP_TARGET_FPS, 60);

    // A frame that finishes inside the 16.67 ms budget passes; one that
    // overruns it does not.
    assert!(renderer.meets_60fps(16.0), "16 ms frame must hit 60 fps");
    assert!(!renderer.meets_60fps(20.0), "20 ms frame exceeds budget");

    // Culling keeps the visible set well below the full map so the frame
    // budget is achievable.
    let draw_list = renderer.build_draw_list();
    assert!(draw_list.visible_tiles > 0, "map must render something");
    assert!(
        (draw_list.visible_tiles as usize) < renderer.tile_count(),
        "off-screen tiles must be culled"
    );
    assert!(draw_list.draw_calls() <= 4, "batched by terrain => few calls");
}
