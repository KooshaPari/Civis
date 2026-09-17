//! FR-ASSET-002 — the asset pipeline SHALL pack all tile sprites into a
//! single texture atlas per LOD level.
//!
//! Matrix check: `asset::atlas_packed_per_lod`.

use civ_render::atlas::{pack_atlas_per_lod, rasterise_at_build, SvgSource};
use civ_render::lod::LodLevel;

fn sprite(name: &str, w: u32, h: u32, lod: u8) -> SvgSource {
    SvgSource {
        name: name.to_string(),
        svg: format!("<svg width=\"{w}\" height=\"{h}\"><rect/></svg>"),
        lod: LodLevel(lod),
    }
}

/// Exactly one atlas is produced per LOD level, sprites are placed inside the
/// atlas bounds, and dimensions are powers of two.
#[test]
fn atlas_packed_per_lod() {
    let sources = vec![
        sprite("grass", 16, 16, 0),
        sprite("sand", 16, 16, 0),
        sprite("forest", 16, 16, 1),
        sprite("hill", 16, 16, 2),
        sprite("peak", 16, 16, 2),
    ];
    let sprites = rasterise_at_build(&sources).expect("rasterises");
    let atlases = pack_atlas_per_lod(&sprites).expect("packs");

    // One atlas per distinct LOD level, ordered by level.
    assert_eq!(atlases.len(), 3, "LOD 0, 1, 2 => three atlases");
    let lods: Vec<u8> = atlases.iter().map(|a| a.lod.0).collect();
    assert_eq!(lods, vec![0, 1, 2]);

    // Every sprite appears exactly once, in its own LOD's atlas.
    for lod in 0..3u8 {
        let atlas = atlases.iter().find(|a| a.lod == LodLevel(lod)).unwrap();
        let expected = sources.iter().filter(|s| s.lod == LodLevel(lod)).count();
        assert_eq!(atlas.entries.len(), expected, "all sprites of LOD {lod}");
        for entry in &atlas.entries {
            assert_eq!(entry.w, 16);
            assert_eq!(entry.h, 16);
            // Frame is fully contained within the atlas.
            assert!(entry.x + entry.w <= atlas.width, "no horizontal overflow");
            assert!(entry.y + entry.h <= atlas.height, "no vertical overflow");
        }
    }

    // Atlas dimensions are powers of two and pixels are sized to match.
    for atlas in &atlases {
        assert!(atlas.width.is_power_of_two(), "width must be pow2");
        assert!(atlas.height.is_power_of_two(), "height must be pow2");
        assert_eq!(
            atlas.pixels.len(),
            (atlas.width * atlas.height * 4) as usize,
            "RGBA8 buffer sized to atlas"
        );
        assert!(atlas.area() >= 16 * 16);
    }
}
