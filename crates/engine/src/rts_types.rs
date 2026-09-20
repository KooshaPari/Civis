//! RTS rendering, nation colors, and atlas types (FR-CIV-RTS-*).
//!
//! Types for the 2D asset pipeline and runtime sprite system described in
//! `docs/specs/CIV-0600-2d-asset-pipeline-spec.md`. These cover nation color
//! binding, atlas configuration, zoom-level switching, and sprite management.

use serde::{Deserialize, Serialize};

// ── Nation color types (§6.3, §10.3 of CIV-0600) ─────────────────────────────

/// A nation's color palette for sprite recoloring.
// FR-CIV-RTS-RENDER-003
/// Nation colors are not baked into atlas sprites; a fragment shader
/// replaces palette indices at render time.
// FR-CIV-ASSET-003
// FR-CIV-ASSET-004
// FR-CIV-ASSET-005
// FR-CIV-ASSET-016
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NationColor {
    /// Primary color as hex string (e.g. "#c8303c").
    pub primary_hex: String,
    /// Secondary/accent color as hex string (e.g. "#f0c040").
    pub secondary_hex: String,
    /// Darkened primary (borders, outlines) as hex string.
    pub dark_hex: String,
}

impl NationColor {
    /// Parse a hex color string like "#c8303c" into [r, g, b].
    pub fn parse_hex(hex: &str) -> Option<[u8; 3]> {
        let h = hex.trim_start_matches('#');
        if h.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&h[0..2], 16).ok()?;
        let g = u8::from_str_radix(&h[2..4], 16).ok()?;
        let b = u8::from_str_radix(&h[4..6], 16).ok()?;
        Some([r, g, b])
    }

    /// Convert primary color to RGBA [r, g, b, 255] for shader uniform.
    pub fn primary_rgba(&self) -> Option<[u8; 4]> {
        Self::parse_hex(&self.primary_hex).map(|[r, g, b]| [r, g, b, 255])
    }

    /// Convert secondary color to RGBA [r, g, b, 255] for shader uniform.
    pub fn secondary_rgba(&self) -> Option<[u8; 4]> {
        Self::parse_hex(&self.secondary_hex).map(|[r, g, b]| [r, g, b, 255])
    }

    /// Default nation palette: forced at indices 0 and 1 during quantization.
    pub const PALETTE_INDEX_PRIMARY: usize = 0;
    pub const PALETTE_INDEX_SECONDARY: usize = 1;
}

/// Default baked colors that the SVG templates use. The shader replaces
/// these with the actual nation colors at render time.
pub const BAKED_PRIMARY: &str = "#c8303c";
pub const BAKED_SECONDARY: &str = "#f0c040";

// ── Atlas configuration (§7.2) ───────────────────────────────────────────────

// FR-CIV-RTS-RENDER-004
/// Atlas dimensions are fixed by asset category. Power-of-two required
/// for WebGL texture compatibility.
// FR-CIV-ASSET-006
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AtlasConfig {
    /// Atlas name (e.g. "terrain_atlas").
    pub name: String,
    /// Width in pixels (must be power of two).
    pub width: u32,
    /// Height in pixels (must be power of two).
    pub height: u32,
    /// Sprite size for this atlas at its primary zoom level.
    pub sprite_size: u32,
}

impl AtlasConfig {
    /// Terrain atlas: 2048x2048.
    pub fn terrain() -> Self {
        Self {
            name: "terrain_atlas".into(),
            width: 2048,
            height: 2048,
            sprite_size: 64, // Z1
        }
    }

    /// Buildings atlas: 1024x1024.
    pub fn buildings() -> Self {
        Self {
            name: "buildings_atlas".into(),
            width: 1024,
            height: 1024,
            sprite_size: 128, // Z2
        }
    }

    /// Citizens atlas: 512x512.
    pub fn citizens() -> Self {
        Self {
            name: "citizens_atlas".into(),
            width: 512,
            height: 512,
            sprite_size: 64, // Z3
        }
    }

    /// Check if both dimensions are power-of-two.
    pub fn is_pow2(&self) -> bool {
        self.width.is_power_of_two() && self.height.is_power_of_two()
    }

    /// Total pixel area.
    pub fn area(&self) -> u64 {
        self.width as u64 * self.height as u64
    }
}

/// All three atlas configs in the pipeline.
pub fn all_atlases() -> Vec<AtlasConfig> {
    vec![
        AtlasConfig::terrain(),
        AtlasConfig::buildings(),
        AtlasConfig::citizens(),
    ]
}

// ── Zoom levels and atlas mapping (§1.3, §10.2) ─────────────────────────────

/// Three zoom levels for the RTS client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZoomTier {
    /// Strategic / Nation view (Zoom 1) — 64x64 sprites.
    Strategic,
    /// Tactical / City view (Zoom 2) — 128x128 sprites.
    Tactical,
    /// Citizen / Research view (Zoom 3) — 64x64 sprites.
    Citizen,
}

impl ZoomTier {
    /// Sprite size in pixels for this zoom level.
    pub fn sprite_size(self) -> u32 {
        match self {
            Self::Strategic => 64,
            Self::Tactical => 128,
            Self::Citizen => 64,
        }
    }

    /// Atlas name used at this zoom level.
    pub fn atlas_name(self) -> &'static str {
        match self {
            Self::Strategic => "terrain_atlas",
            Self::Tactical => "buildings_atlas",
            Self::Citizen => "citizens_atlas",
        }
    }

    /// The zoom-level suffix for asset IDs (e.g. "_z1").
    pub fn suffix(self) -> &'static str {
        match self {
            Self::Strategic => "_z1",
            Self::Tactical => "_z2",
            Self::Citizen => "_z3",
        }
    }
}

// FR-CIV-RTS-ZOOM-001
/// A sprite handle that references a specific asset at a zoom level.
// FR-CIV-ASSET-018
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteHandle {
    /// Asset identifier (e.g. "terrain_plains").
    pub asset_id: String,
    /// Current zoom level.
    pub zoom: ZoomTier,
    /// Whether the texture has been swapped this frame (for swap tracking).
    pub swapped_this_frame: bool,
}

impl SpriteHandle {
    /// Create a new sprite handle at the given zoom level.
    pub fn new(asset_id: &str, zoom: ZoomTier) -> Self {
        Self {
            asset_id: asset_id.to_string(),
            zoom,
            swapped_this_frame: false,
        }
    }

    /// Get the texture key including zoom suffix.
    pub fn texture_key(&self) -> String {
        format!("{}{}", self.asset_id, self.zoom.suffix())
    }

    /// Swap the zoom level (simulating atlas texture swap).
    pub fn set_zoom(&mut self, new_zoom: ZoomTier) {
        self.zoom = new_zoom;
        self.swapped_this_frame = true;
    }
}

// ── UV rect (§7.3) ───────────────────────────────────────────────────────────

// FR-CIV-RTS-RENDER-005
/// Rectangle within an atlas, used for UV mapping.
// FR-CIV-ASSET-007
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UvRect {
    /// X offset in pixels.
    pub x: u32,
    /// Y offset in pixels.
    pub y: u32,
    /// Width in pixels.
    pub w: u32,
    /// Height in pixels.
    pub h: u32,
}

impl UvRect {
    /// Check whether this rect fits inside the given atlas dimensions.
    pub fn fits_in(&self, atlas_w: u32, atlas_h: u32) -> bool {
        self.x + self.w <= atlas_w && self.y + self.h <= atlas_h
    }

    /// Check whether two rects overlap (exclusive — touching edges are not overlap).
    pub fn overlaps(&self, other: &UvRect) -> bool {
        self.x < other.x + other.w
            && self.x + self.w > other.x
            && self.y < other.y + other.h
            && self.y + self.h > other.y
    }

    /// Area in pixels.
    pub fn area(&self) -> u64 {
        self.w as u64 * self.h as u64
    }
}

// ── Supersampling (§3.2) ─────────────────────────────────────────────────────

// FR-CIV-RTS-RENDER-001, FR-CIV-RTS-RENDER-002
/// Supersampling configuration for sprite rasterization.
// FR-CIV-ASSET-001
// FR-CIV-ASSET-003
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SsConfig {
    /// Supersampling factor (e.g. 4 for 4x).
    pub factor: u32,
    /// Final output width.
    pub output_width: u32,
    /// Final output height.
    pub output_height: u32,
}

impl SsConfig {
    /// Create a config for 4x supersampling.
    pub fn four_x(output_w: u32, output_h: u32) -> Self {
        Self {
            factor: 4,
            output_width: output_w,
            output_height: output_h,
        }
    }

    /// The internal render width before downscale.
    pub fn render_width(&self) -> u32 {
        self.output_width * self.factor
    }

    /// The internal render height before downscale.
    pub fn render_height(&self) -> u32 {
        self.output_height * self.factor
    }
}

// ── Nation recoloring shader distance (§10.3) ────────────────────────────────

/// Color matching tolerance for the nation recoloring shader.
/// The shader uses a distance check with this tolerance to handle
/// dithering artifacts from palette quantization.
pub const SHADER_TOLERANCE: f32 = 0.08;

/// Compute the Euclidean distance between two RGB colors (0..1 range).
pub fn color_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dr = a[0] - b[0];
    let dg = a[1] - b[1];
    let db = a[2] - b[2];
    (dr * dr + dg * dg + db * db).sqrt()
}

/// Check whether two RGB colors match within the shader tolerance.
pub fn color_matches(a: [f32; 3], b: [f32; 3]) -> bool {
    color_distance(a, b) < SHADER_TOLERANCE
}
