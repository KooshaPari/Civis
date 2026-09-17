//! SVG build-time rasterisation and texture atlas packing (CIV-0600).
//!
//! Implements:
//!
//! - **FR-ASSET-001** — All 2D tile sprites SHALL be derived from SVG sources
//!   and rasterised at build time. [`rasterise_at_build`] performs that pass:
//!   it only ever consumes SVG text, never pre-baked bitmaps.
//! - **FR-ASSET-002** — The asset pipeline SHALL pack all tile sprites into a
//!   single texture atlas per LOD level. [`pack_atlas_per_lod`] emits exactly
//!   one [`TextureAtlas`] per [`LodLevel`] present in the sprite set.
//! - **FR-ASSET-003** — Atlas build SHALL emit `asset.atlas.built.v1` on
//!   success or `asset.generation.failed.v1` on error.
//!   [`atlas_build_event`] produces that payload for the event bus.
//!
//! Rasterisation here is deterministic and CPU-only: sprite geometry comes from
//! the SVG's `width`/`height` attributes and the fill colour is derived from a
//! hash of the SVG body, so the same source always yields the same pixels. The
//! real renderer swaps in `resvg`/`tiny-skia` behind the same signature.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::lod::LodLevel;

/// Event type emitted when atlases build successfully.
pub const EVENT_ATLAS_BUILT: &str = "asset.atlas.built.v1";

/// Event type emitted when asset generation fails.
pub const EVENT_GENERATION_FAILED: &str = "asset.generation.failed.v1";

/// Default sprite edge length when an SVG omits explicit dimensions.
pub const DEFAULT_SPRITE_SIZE: u32 = 16;

/// A sprite's SVG source, tagged with the LOD it belongs to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SvgSource {
    /// Stable sprite name (also the atlas entry key).
    pub name: String,
    /// Raw SVG document text.
    pub svg: String,
    /// LOD bucket this sprite is authored for.
    pub lod: LodLevel,
}

/// A rasterised sprite: tightly packed RGBA8 pixels.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RasterSprite {
    /// Sprite name.
    pub name: String,
    /// LOD bucket.
    pub lod: LodLevel,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Row-major RGBA8 buffer of `width * height * 4` bytes.
    pub rgba: Vec<u8>,
}

/// Errors surfaced by the asset pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetError {
    /// The sprite set was empty; there is nothing to rasterise or pack.
    NoSources,
    /// An SVG source had an empty body.
    EmptySvg {
        /// Name of the offending sprite.
        name: String,
    },
}

impl AssetError {
    /// Stable machine-readable error code for the failure event.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::NoSources => "no_sources",
            Self::EmptySvg { .. } => "empty_svg",
        }
    }
}

/// Extract an integer SVG presentation attribute (e.g. `width="24"`).
fn parse_dim(svg: &str, attr: &str) -> Option<u32> {
    let needle = format!("{attr}=\"");
    let start = svg.find(&needle)? + needle.len();
    let rest = &svg[start..];
    let end = rest.find('"')?;
    let raw = rest[..end].trim_end_matches(|c: char| !c.is_ascii_digit());
    raw.parse::<u32>().ok().filter(|n| *n > 0)
}

/// Deterministic 32-bit FNV-1a hash of the SVG body, used to derive fill colour.
fn svg_fill(svg: &str) -> [u8; 4] {
    let mut hash: u32 = 0x811c_9dc5;
    for b in svg.as_bytes() {
        hash ^= u32::from(*b);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    [
        (hash & 0xff) as u8,
        ((hash >> 8) & 0xff) as u8,
        ((hash >> 16) & 0xff) as u8,
        ((hash >> 24) & 0xff) as u8,
    ]
}

/// Rasterise SVG sources at build time (FR-ASSET-001).
///
/// # Errors
///
/// Returns [`AssetError::NoSources`] for an empty slice and
/// [`AssetError::EmptySvg`] when a source's body is blank.
pub fn rasterise_at_build(sources: &[SvgSource]) -> Result<Vec<RasterSprite>, AssetError> {
    if sources.is_empty() {
        return Err(AssetError::NoSources);
    }

    let mut out = Vec::with_capacity(sources.len());
    for src in sources {
        if src.svg.trim().is_empty() {
            return Err(AssetError::EmptySvg {
                name: src.name.clone(),
            });
        }
        let width = parse_dim(&src.svg, "width").unwrap_or(DEFAULT_SPRITE_SIZE);
        let height = parse_dim(&src.svg, "height").unwrap_or(DEFAULT_SPRITE_SIZE);
        let fill = svg_fill(&src.svg);
        let mut rgba = Vec::with_capacity((width * height * 4) as usize);
        for _ in 0..(width * height) {
            rgba.extend_from_slice(&fill);
        }
        out.push(RasterSprite {
            name: src.name.clone(),
            lod: src.lod,
            width,
            height,
            rgba,
        });
    }
    Ok(out)
}

/// One sprite's placement inside an atlas.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AtlasEntry {
    /// Sprite name.
    pub name: String,
    /// Left offset in the atlas, in pixels.
    pub x: u32,
    /// Top offset in the atlas, in pixels.
    pub y: u32,
    /// Width in pixels.
    pub w: u32,
    /// Height in pixels.
    pub h: u32,
}

/// A single packed texture atlas for one LOD level (FR-ASSET-002).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextureAtlas {
    /// LOD level this atlas serves.
    pub lod: LodLevel,
    /// Atlas width in pixels (power of two).
    pub width: u32,
    /// Atlas height in pixels (power of two).
    pub height: u32,
    /// Placement of each sprite.
    pub entries: Vec<AtlasEntry>,
    /// Packed RGBA8 atlas pixels.
    pub pixels: Vec<u8>,
}

impl TextureAtlas {
    /// Pixel area of the atlas.
    #[must_use]
    pub fn area(&self) -> u64 {
        u64::from(self.width) * u64::from(self.height)
    }
}

fn next_pow2(n: u32) -> u32 {
    let mut p = 1;
    while p < n {
        p <<= 1;
    }
    p
}

/// Pack sprites into one atlas per LOD level (FR-ASSET-002).
///
/// Sprites are grouped by [`LodLevel`] and shelf-packed within each group; the
/// resulting atlas dimensions are rounded up to the next power of two.
///
/// # Errors
///
/// Returns [`AssetError::NoSources`] when `sprites` is empty.
pub fn pack_atlas_per_lod(sprites: &[RasterSprite]) -> Result<Vec<TextureAtlas>, AssetError> {
    if sprites.is_empty() {
        return Err(AssetError::NoSources);
    }

    let mut lods: Vec<LodLevel> = sprites.iter().map(|s| s.lod).collect();
    lods.sort_unstable();
    lods.dedup();

    let mut atlases = Vec::with_capacity(lods.len());
    for lod in lods {
        let group: Vec<&RasterSprite> = sprites.iter().filter(|s| s.lod == lod).collect();

        // Shelf packing: fill a row until the width cap, then start a new row.
        let max_row_width = 256u32;
        let mut x = 0u32;
        let mut y = 0u32;
        let mut row_h = 0u32;
        let mut used_w = 0u32;
        let mut entries = Vec::with_capacity(group.len());

        for sprite in &group {
            if x > 0 && x + sprite.width > max_row_width {
                x = 0;
                y += row_h;
                row_h = 0;
            }
            entries.push(AtlasEntry {
                name: sprite.name.clone(),
                x,
                y,
                w: sprite.width,
                h: sprite.height,
            });
            x += sprite.width;
            used_w = used_w.max(x);
            row_h = row_h.max(sprite.height);
        }
        let used_h = y + row_h;

        let width = next_pow2(used_w.max(1));
        let height = next_pow2(used_h.max(1));
        let mut pixels = vec![0u8; (width * height * 4) as usize];

        for (entry, sprite) in entries.iter().zip(group.iter()) {
            for row in 0..sprite.height {
                let dst = (((entry.y + row) * width + entry.x) * 4) as usize;
                let src = (row * sprite.width * 4) as usize;
                let len = (sprite.width * 4) as usize;
                pixels[dst..dst + len].copy_from_slice(&sprite.rgba[src..src + len]);
            }
        }

        atlases.push(TextureAtlas {
            lod,
            width,
            height,
            entries,
            pixels,
        });
    }
    Ok(atlases)
}

/// Build the event payload for an atlas build attempt (FR-ASSET-003).
///
/// Emits [`EVENT_ATLAS_BUILT`] on success (with the LOD levels produced) or
/// [`EVENT_GENERATION_FAILED`] on error (with the failure code).
#[must_use]
pub fn atlas_build_event(result: &Result<Vec<TextureAtlas>, AssetError>) -> Value {
    match result {
        Ok(atlases) => {
            let lods: Vec<u8> = atlases.iter().map(|a| a.lod.0).collect();
            let sprites: usize = atlases.iter().map(|a| a.entries.len()).sum();
            json!({
                "event_type": EVENT_ATLAS_BUILT,
                "atlas_count": atlases.len(),
                "lods": lods,
                "sprite_count": sprites,
            })
        }
        Err(err) => json!({
            "event_type": EVENT_GENERATION_FAILED,
            "stage": "atlas",
            "code": err.code(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn src(name: &str, w: u32, h: u32, lod: u8) -> SvgSource {
        SvgSource {
            name: name.to_string(),
            svg: format!("<svg width=\"{w}\" height=\"{h}\"><rect/></svg>"),
            lod: LodLevel(lod),
        }
    }

    #[test]
    fn rasterises_from_svg_dimensions() {
        let sprites = rasterise_at_build(&[src("grass", 8, 4, 0)]).unwrap();
        assert_eq!(sprites.len(), 1);
        assert_eq!(sprites[0].width, 8);
        assert_eq!(sprites[0].height, 4);
        assert_eq!(sprites[0].rgba.len(), 8 * 4 * 4);
    }

    #[test]
    fn empty_input_errors() {
        assert_eq!(rasterise_at_build(&[]), Err(AssetError::NoSources));
        assert_eq!(pack_atlas_per_lod(&[]), Err(AssetError::NoSources));
    }

    #[test]
    fn empty_svg_body_errors() {
        let bad = SvgSource {
            name: "bad".into(),
            svg: "   ".into(),
            lod: LodLevel(0),
        };
        assert_eq!(
            rasterise_at_build(&[bad]),
            Err(AssetError::EmptySvg { name: "bad".into() })
        );
    }

    #[test]
    fn one_atlas_per_lod() {
        let sprites = rasterise_at_build(&[
            src("a", 8, 8, 0),
            src("b", 8, 8, 1),
            src("c", 8, 8, 1),
        ])
        .unwrap();
        let atlases = pack_atlas_per_lod(&sprites).unwrap();
        assert_eq!(atlases.len(), 2, "one atlas per distinct LOD");
        assert_eq!(atlases[0].lod, LodLevel(0));
        assert_eq!(atlases[1].lod, LodLevel(1));
        assert_eq!(atlases[1].entries.len(), 2);
    }

    #[test]
    fn atlas_dims_are_power_of_two_and_pixels_sized() {
        let sprites = rasterise_at_build(&[src("a", 5, 5, 0), src("b", 5, 5, 0)]).unwrap();
        let atlases = pack_atlas_per_lod(&sprites).unwrap();
        let a = &atlases[0];
        assert!(a.width.is_power_of_two());
        assert!(a.height.is_power_of_two());
        assert_eq!(a.pixels.len(), (a.width * a.height * 4) as usize);
    }

    #[test]
    fn built_event_on_success_failed_event_on_error() {
        let atlases = pack_atlas_per_lod(&rasterise_at_build(&[src("a", 4, 4, 0)]).unwrap())
            .unwrap();
        let ok = atlas_build_event(&Ok(atlases));
        assert_eq!(ok["event_type"], EVENT_ATLAS_BUILT);

        let bad = atlas_build_event(&Err(AssetError::NoSources));
        assert_eq!(bad["event_type"], EVENT_GENERATION_FAILED);
        assert_eq!(bad["code"], "no_sources");
    }
}
