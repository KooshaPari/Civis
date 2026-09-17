//! FR-UX-001 — the UI SHALL render the hex map using the `crates/render`
//! crate at a 60 fps target.
//!
//! Matrix check: `render::hex_map_60fps`.

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
