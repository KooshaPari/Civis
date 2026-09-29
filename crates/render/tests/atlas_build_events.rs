//!
//! Provenance: this test previously carried a `FR-ASSET-003` tag. No authoritative
//! spec defines that id; its only definition was a table in
//! `docs/traceability/TRACEABILITY_MATRIX.md` citing a nonexistent spec
//! file. The assertions below are real and are retained as a behavioral test
//! of atlas build event payloads; the false id was removed rather than rebound. See
//! `docs/audits/id-provenance-corrections.md`.

use civ_render::atlas::{
    atlas_build_event, pack_atlas_per_lod, rasterise_at_build, AssetError, SvgSource,
    EVENT_ATLAS_BUILT, EVENT_GENERATION_FAILED,
};
use civ_render::lod::LodLevel;

fn sprite(name: &str, lod: u8) -> SvgSource {
    SvgSource {
        name: name.to_string(),
        svg: "<svg width=\"8\" height=\"8\"><rect/></svg>".into(),
        lod: LodLevel(lod),
    }
}

/// The exact event type strings are emitted, with success and failure payloads.
#[test]
fn atlas_build_events() {
    assert_eq!(EVENT_ATLAS_BUILT, "asset.atlas.built.v1");
    assert_eq!(EVENT_GENERATION_FAILED, "asset.generation.failed.v1");

    // Success path.
    let atlases = pack_atlas_per_lod(
        &rasterise_at_build(&[sprite("grass", 0), sprite("hill", 1)]).unwrap(),
    )
    .unwrap();
    let built = atlas_build_event(&Ok(atlases));
    assert_eq!(built["event_type"], EVENT_ATLAS_BUILT);
    assert_eq!(built["atlas_count"], 2);
    assert_eq!(built["sprite_count"], 2);

    // Error path.
    let failed = atlas_build_event(&Err(AssetError::NoSources));
    assert_eq!(failed["event_type"], EVENT_GENERATION_FAILED);
    assert_eq!(failed["stage"], "atlas");
    assert_eq!(failed["code"], "no_sources");

    let empty_svg = atlas_build_event(&Err(AssetError::EmptySvg {
        name: "bad".into(),
    }));
    assert_eq!(empty_svg["event_type"], EVENT_GENERATION_FAILED);
    assert_eq!(empty_svg["code"], "empty_svg");

    // Success and failure are mutually exclusive: exactly one event type fires.
    assert_ne!(built["event_type"], failed["event_type"]);
}
