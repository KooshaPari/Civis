//! FR-ASSET-001 — all 2D tile sprites SHALL be derived from SVG sources and
//! rasterised at build time.
//!
//! Matrix check: `asset::svg_rasterised_at_build`.

use civ_render::atlas::{rasterise_at_build, AssetError, SvgSource, DEFAULT_SPRITE_SIZE};
use civ_render::lod::LodLevel;

fn src(name: &str, svg: String, lod: u8) -> SvgSource {
    SvgSource {
        name: name.to_string(),
        svg,
        lod: LodLevel(lod),
    }
}

/// Sprites come only from SVG text, and rasterisation is a deterministic
/// build-time step.
#[test]
fn svg_rasterised_at_build() {
    let sprites = rasterise_at_build(&[
        src("grass", "<svg width=\"32\" height=\"32\"><rect/></svg>".into(), 0),
        src("tree", "<svg width=\"24\" height=\"40\"><path/></svg>".into(), 0),
    ])
    .expect("valid SVG sources rasterise");

    assert_eq!(sprites.len(), 2);
    assert_eq!(sprites[0].width, 32);
    assert_eq!(sprites[0].height, 32);
    assert_eq!(sprites[1].width, 24);
    assert_eq!(sprites[1].height, 40);

    // Pixels are RGBA8 and sized exactly to width * height.
    for s in &sprites {
        assert_eq!(s.rgba.len(), (s.width * s.height * 4) as usize);
    }

    // Rasterisation is deterministic: identical SVG => identical pixels.
    let again = rasterise_at_build(&[
        src("grass", "<svg width=\"32\" height=\"32\"><rect/></svg>".into(), 0),
        src("tree", "<svg width=\"24\" height=\"40\"><path/></svg>".into(), 0),
    ])
    .expect("re-rasterises");
    assert_eq!(sprites, again, "build-time raster must be reproducible");

    // Missing dimensions fall back to the default sprite size.
    let fallback = rasterise_at_build(&[src("anon", "<svg><circle/></svg>".into(), 0)])
        .expect("rasterises with defaults");
    assert_eq!(fallback[0].width, DEFAULT_SPRITE_SIZE);

    // A pre-baked bitmap path cannot satisfy this FR: only SVG text is accepted,
    // and empty sources are rejected rather than silently producing blanks.
    assert_eq!(rasterise_at_build(&[]), Err(AssetError::NoSources));
    assert_eq!(
        rasterise_at_build(&[src("blank", "   ".into(), 0)]),
        Err(AssetError::EmptySvg { name: "blank".into() })
    );
}
