//! Greedy 2D bin-packing atlas for triplanar PBR texture layers.
//!
//! Packs a list of heterogeneous `[AtlasTexture]` rectangles into a single
//! 2D atlas of power-of-two dimensions. The algorithm is a textbook
//! **greedy shelf packer**: sort textures by descending height, then sweep
//! the atlas left-to-right and bottom-up, dropping each rectangle onto the
//! first shelf it fits in. Oversized textures that do not fit any shelf
//! cause [`GreedyAtlas::pack`] to fail with [`AtlasError::RectTooLarge`].
//!
//! The packer is engine-agnostic and lives in `civ-voxel` so the Bevy-gated
//! adapter (`clients/bevy-ref/src/materials.rs`) can call into it without
//! pulling `bevy` into the substrate crate. The `AtlasRect` returned for
//! each input texture feeds straight into the triplanar WGSL shader via
//! the `TriplanarPbrMaterial` (see `triplanar_pipeline.rs`).
//!
//! # Determinism
//!
//! Iteration order is the sorted `(height desc, id desc)` order. Inputs with
//! the same `id` are rejected, so two calls with the same input list produce
//! identical layouts (replay-safe).
//!
//! # FR coverage
//!
//! - [`AtlasRect::uv`] returns `[Vec2; 4]` corner UVs for the four-vertex
//!   quad bind used by `bevy_pbr::StandardMaterial`.
//! - [`GreedyAtlas::pack`] invokes the greedy shelf algorithm in `O(n log n)`
//!   for sort + `O(n * s)` where `s` is the number of shelves (worst case `n`).
//! - [`AtlasError`] enumerates every failure path so callers do not have to
//!   parse string messages.

#![forbid(unsafe_code)]

use std::fmt;

/// One input rectangle to pack. `id` MUST be unique across a single `pack`
/// call — duplicate ids are rejected as [`AtlasError::DuplicateId`] so the
/// output mapping is unambiguous.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AtlasTexture {
    /// Stable identifier the caller uses to retrieve its packed rectangle.
    /// Typically a `MaterialId` (`u16`) or a content hash.
    pub id: u32,
    /// Width in texels (0..width).
    pub width: u16,
    /// Height in texels (0..height).
    pub height: u16,
}

impl AtlasTexture {
    /// Construct a new texture. `width` and `height` must both be > 0; the
    /// caller is responsible for any power-of-two constraint — this packer
    /// accepts arbitrary sizes (though binary-search-friendly sizes pack
    /// better).
    #[must_use]
    pub const fn new(id: u32, width: u16, height: u16) -> Self {
        Self { id, width, height }
    }
}

/// Packed rectangle in the atlas, returned in input order by [`GreedyAtlas::pack`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AtlasRect {
    /// Identifier matching one of the input [`AtlasTexture::id`]s.
    pub id: u32,
    /// X coordinate of the rectangle's top-left corner, in texels.
    pub x: u16,
    /// Y coordinate of the rectangle's top-left corner, in texels.
    pub y: u16,
    /// Width in texels (== input `width`).
    pub width: u16,
    /// Height in texels (== input `height`).
    pub height: u16,
}

impl AtlasRect {
    /// Sample UV in the `[0, 1]` range for a given texel offset within this
    /// rect. Does NOT add the atlas-layer offset; multiply by `layer_inv_size`
    /// (= `1.0 / ATLAS_SIZE`) on the caller side to get the position in atlas
    /// space.
    #[must_use]
    pub fn uv_at(&self, lx: f32, ly: f32, atlas_size: f32) -> [Vec2; 4] {
        debug_assert!(atlas_size > 0.0);
        let ax = (f32::from(self.x) + lx * f32::from(self.width)) / atlas_size;
        let ay = (f32::from(self.y) + ly * f32::from(self.height)) / atlas_size;
        let corners = [
            (0.0, 0.0),
            (1.0, 0.0),
            (1.0, 1.0),
            (0.0, 1.0),
        ];
        corners.map(|(u, v)| Vec2 {
            x: ax + u * f32::from(self.width) / atlas_size,
            y: ay + v * f32::from(self.height) / atlas_size,
        })
    }

    /// Four corner UVs (top-left, top-right, bottom-right, bottom-left)
    /// for this rect on the given atlas layer. `layer` is a 0-based layer
    /// index; the caller multiplies it by `1.0/atlas_depth` and offsets
    /// into the third texture coordinate.
    #[must_use]
    pub fn uv(&self, _layer: u16, atlas_size: f32) -> [Vec2; 4] {
        // The layer index only affects the W-coordinate in `texture_2d_array`
        // sampling; 2D quad corner UVs are layer-independent. We accept the
        // argument so the call site reads symmetric to the rest of the pipeline
        // (and a future layer-atlas variant can fold it in).
        self.uv_at(0.0, 0.0, atlas_size)
    }
}

/// Lightweight 2D vector type used in the public API. We avoid pulling in
/// `glam` here so this module can compile in `no_std`-friendly contexts if
/// the substrate ever moves that way.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

/// A single horizontal shelf inside the atlas. Rectangles packed onto a shelf
/// share its `y..y+shelf_h` range; new rectangles are placed at the rightmost
/// free x-position.
#[derive(Debug, Clone, Copy)]
struct Shelf {
    /// Y coordinate of the shelf's top edge.
    y: u16,
    /// Height of the shelf in texels.
    height: u16,
    /// Current right-edge x — next rectangle sits at this x.
    cursor_x: u16,
}

/// Errors returned by [`GreedyAtlas::pack`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AtlasError {
    /// Two or more [`AtlasTexture`]s in the input shared the same id.
    DuplicateId(u32),
    /// A rectangle's width or height exceeded the atlas dimension — there
    /// is no shelf high or wide enough to hold it.
    RectTooLarge {
        /// Offending rectangle id.
        id: u32,
        /// Requested width.
        width: u16,
        /// Requested height.
        height: u16,
    },
    /// A rectangle was rejected because its width or height was zero.
    ZeroSize { id: u32 },
}

impl fmt::Display for AtlasError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateId(id) => write!(f, "duplicate atlas id {id}"),
            Self::RectTooLarge { id, width, height } => write!(
                f,
                "rect {id} ({width}x{height}) does not fit in the atlas"
            ),
            Self::ZeroSize { id } => write!(f, "rect {id} has zero width or height"),
        }
    }
}

impl std::error::Error for AtlasError {}

/// Greedy 2D bin-packing atlas. Holds the atlas dimensions plus the working
/// set of shelves returned by the most recent [`Self::pack`] call.
#[derive(Debug, Clone)]
pub struct GreedyAtlas {
    /// Atlas width in texels. Must be a power of two for GPU-friendly sampling.
    pub width: u32,
    /// Atlas height in texels. Must be a power of two for GPU-friendly sampling.
    pub height: u32,
    /// Internal shelves rebuilt on every `pack` call.
    shelves: Vec<Shelf>,
}

impl GreedyAtlas {
    /// Construct a new atlas of the given dimensions. `width` and `height`
    /// must each be a power of two (the packer does NOT enforce this — it is
    /// the caller's responsibility to satisfy GPU mipmapping constraints).
    ///
    /// # Panics
    /// Panics if `width == 0` or `height == 0`. The contract is documented; an
    /// explicit panic surfaces the misuse at construction time.
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        assert!(width > 0, "atlas width must be > 0");
        assert!(height > 0, "atlas height must be > 0");
        Self { width, height, shelves: Vec::new() }
    }

    /// Pack a slice of textures into the atlas. Returns one [`AtlasRect`] per
    /// input, in the same order as the input slice. On failure returns the
    /// first [`AtlasError`] encountered — no partial packing is reported.
    ///
    /// The shelf packer is:
    ///
    /// 1. Reject duplicate ids (sorted scan).
    /// 2. Reject any `width == 0 || height == 0` rect.
    /// 3. Sort textures by `(height desc, id desc)` — descending height gives
    ///    a tighter packing than descending width on average.
    /// 4. For each texture, scan the shelf list; if any shelf can hold it
    ///    (`cursor_x + width <= atlas.width` and `shelf.height >= height`),
    ///    drop it on the first such shelf. Otherwise allocate a new shelf at
    ///    `y = sum(shelf heights so far)` if it fits within `atlas.height`.
    /// 5. On a new shelf overflow, return [`AtlasError::RectTooLarge`].
    pub fn pack(&mut self, textures: &[AtlasTexture]) -> Result<Vec<AtlasRect>, AtlasError> {
        self.shelves.clear();

        if textures.is_empty() {
            return Ok(Vec::new());
        }

        // Step 1: duplicate-id detection.
        let mut sorted_ids: Vec<u32> = textures.iter().map(|t| t.id).collect();
        sorted_ids.sort_unstable();
        for window in sorted_ids.windows(2) {
            if window[0] == window[1] {
                return Err(AtlasError::DuplicateId(window[0]));
            }
        }

        // Step 2: zero-size rejection.
        for t in textures {
            if t.width == 0 || t.height == 0 {
                return Err(AtlasError::ZeroSize { id: t.id });
            }
        }

        // Step 3: descending-height sort, tie-break by id descending.
        let mut sorted: Vec<AtlasTexture> = textures.to_vec();
        sorted.sort_by(|a, b| {
            b.height
                .cmp(&a.height)
                .then_with(|| b.id.cmp(&a.id))
        });

        // Step 4 + 5: sweep pack.
        let mut placed: std::collections::HashMap<u32, AtlasRect> =
            std::collections::HashMap::with_capacity(sorted.len());
        for t in &sorted {
            let w = t.width;
            let h = t.height;

            if u32::from(w) > self.width || u32::from(h) > self.height {
                return Err(AtlasError::RectTooLarge {
                    id: t.id,
                    width: w,
                    height: h,
                });
            }

            // Try to fit on an existing shelf.
            let mut chosen: Option<usize> = None;
            for (i, shelf) in self.shelves.iter().enumerate() {
                if shelf.height >= h
                    && u32::from(shelf.cursor_x) + u32::from(w) <= self.width
                {
                    chosen = Some(i);
                    break;
                }
            }

            let rect = if let Some(i) = chosen {
                let shelf = &mut self.shelves[i];
                let rect = AtlasRect {
                    id: t.id,
                    x: shelf.cursor_x,
                    y: shelf.y,
                    width: w,
                    height: h,
                };
                shelf.cursor_x = shelf.cursor_x.saturating_add(w);
                rect
            } else {
                // Allocate a new shelf at the current bottom.
                let shelf_bottom: u32 =
                    self.shelves.iter().map(|s| u32::from(s.y) + u32::from(s.height)).sum();
                if shelf_bottom + u32::from(h) > self.height {
                    return Err(AtlasError::RectTooLarge {
                        id: t.id,
                        width: w,
                        height: h,
                    });
                }
                let rect = AtlasRect {
                    id: t.id,
                    x: 0,
                    y: shelf_bottom as u16,
                    width: w,
                    height: h,
                };
                self.shelves.push(Shelf {
                    y: rect.y,
                    height: h,
                    cursor_x: w,
                });
                rect
            };
            placed.insert(t.id, rect);
        }

        // Restore the caller's original ordering.
        let mut out = Vec::with_capacity(textures.len());
        for t in textures {
            // SAFETY: every input id got a rect in the placement map.
            let r = placed
                .remove(&t.id)
                .expect("every input id is placed exactly once");
            out.push(r);
        }
        Ok(out)
    }

    /// Number of shelves currently held after the last `pack` call. Useful
    /// for diagnostics and benchmarks.
    #[must_use]
    pub fn shelf_count(&self) -> usize {
        self.shelves.len()
    }

    /// Effective pack height (sum of shelf heights) — how far down the atlas
    /// the packer actually filled.
    #[must_use]
    pub fn packed_height(&self) -> u32 {
        self.shelves
            .iter()
            .map(|s| u32::from(s.height))
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(id: u32, w: u16, h: u16) -> AtlasTexture {
        AtlasTexture { id, width: w, height: h }
    }

    /// Packing an empty input returns an empty rect list and leaves the shelves
    /// empty — the degenerate case the Bevy adapter hits on the first frame.
    #[test]
    fn pack_empty_input_returns_no_rects() {
        let mut atlas = GreedyAtlas::new(512, 512);
        let out = atlas.pack(&[]).expect("empty pack succeeds");
        assert!(out.is_empty());
        assert_eq!(atlas.shelf_count(), 0);
        assert_eq!(atlas.packed_height(), 0);
    }

    /// A single texture fills the (0, 0) corner and creates exactly one shelf.
    #[test]
    fn pack_single_texture_lands_at_origin() {
        let mut atlas = GreedyAtlas::new(512, 512);
        let out = atlas
            .pack(&[rect(1, 256, 128)])
            .expect("single rect packs");
        assert_eq!(out.len(), 1);
        assert_eq!(
            out[0],
            AtlasRect { id: 1, x: 0, y: 0, width: 256, height: 128 }
        );
        assert_eq!(atlas.shelf_count(), 1);
        assert_eq!(atlas.packed_height(), 128);
    }

    /// Multiple same-height textures share a single shelf, laid out left-to-right
    /// in descending-id order (deterministic), starting at (0, 0).
    #[test]
    fn pack_multiple_textures_same_height_share_one_shelf() {
        let mut atlas = GreedyAtlas::new(512, 512);
        let out = atlas
            .pack(&[rect(10, 64, 64), rect(20, 128, 64), rect(30, 32, 64)])
            .expect("three rects on one shelf");

        // Descending-id order at pack time → 30, 20, 10 (all heights equal),
        // placed left to right.
        let map: std::collections::HashMap<u32, AtlasRect> =
            out.iter().copied().map(|r| (r.id, r)).collect();
        let r30 = map[&30];
        let r20 = map[&20];
        let r10 = map[&10];
        assert_eq!(r30.x, 0);
        assert_eq!(r30.width, 32);
        assert_eq!(r20.x, 32);
        assert_eq!(r20.width, 128);
        assert_eq!(r10.x, 160);
        assert_eq!(r10.width, 64);
        // All three on shelf 0.
        for r in [r30, r20, r10] {
            assert_eq!(r.y, 0);
            assert_eq!(r.height, 64);
        }
        assert_eq!(atlas.shelf_count(), 1);
        assert_eq!(atlas.packed_height(), 64);
    }

    /// Mixed heights force the packer onto multiple shelves. The first shelf
    /// holds the tallest rectangles and the next shelf catches the rest.
    #[test]
    fn pack_mixed_heights_uses_multiple_shelves() {
        let mut atlas = GreedyAtlas::new(512, 512);
        let out = atlas
            .pack(&[rect(1, 100, 200), rect(2, 100, 100), rect(3, 100, 50)])
            .expect("mixed-height pack");

        // Sorted (height desc, id desc): (1, h=200), (2, h=100), (3, h=50).
        // Shelf 0 holds rect 1; rect 2 starts a new shelf at y=200; rect 3
        // shares shelf 1 because h=50 ≤ 100.
        let map: std::collections::HashMap<u32, AtlasRect> =
            out.iter().copied().map(|r| (r.id, r)).collect();
        assert_eq!(map[&1].y, 0);
        assert_eq!(map[&1].height, 200);
        assert_eq!(map[&2].y, 200);
        assert_eq!(map[&2].x, 0);
        assert_eq!(map[&3].y, 200);
        assert_eq!(map[&3].x, 100);
        assert_eq!(atlas.shelf_count(), 2);
        assert_eq!(atlas.packed_height(), 300);
    }

    /// Output order matches input order regardless of how the packer sorted
    /// internally — the caller's API contract is "input[i] → out[i]".
    #[test]
    fn pack_preserves_input_order() {
        let mut atlas = GreedyAtlas::new(512, 512);
        let out = atlas
            .pack(&[rect(2, 32, 64), rect(1, 32, 64), rect(3, 32, 32)])
            .expect("order-preserving pack");
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].id, 2);
        assert_eq!(out[1].id, 1);
        assert_eq!(out[2].id, 3);
    }

    /// A rectangle bigger than the atlas on either axis returns `RectTooLarge`
    /// — the Bevy adapter calls this on a misconfigured build and we want a
    /// loud failure mode (matches `material_pbr.rs` FR-008 loud policy).
    #[test]
    fn pack_overflow_returns_rect_too_large() {
        let mut atlas = GreedyAtlas::new(512, 512);
        let err = atlas
            .pack(&[rect(1, 256, 256), rect(2, 1024, 256)])
            .expect_err("oversized rect rejected");
        assert_eq!(
            err,
            AtlasError::RectTooLarge {
                id: 2,
                width: 1024,
                height: 256
            }
        );
    }

    /// A duplicate id is rejected before any packing happens — the Bevy
    /// adapter relies on a one-to-one id→rect mapping for material lookups.
    #[test]
    fn pack_duplicate_id_is_rejected() {
        let mut atlas = GreedyAtlas::new(512, 512);
        let err = atlas
            .pack(&[rect(7, 32, 32), rect(7, 32, 32)])
            .expect_err("duplicate id rejected");
        assert_eq!(err, AtlasError::DuplicateId(7));
    }

    /// A zero-width or zero-height rect is rejected as `ZeroSize`.
    #[test]
    fn pack_zero_size_is_rejected() {
        let mut atlas = GreedyAtlas::new(512, 512);
        let err = atlas
            .pack(&[rect(1, 32, 0)])
            .expect_err("zero height rejected");
        assert_eq!(err, AtlasError::ZeroSize { id: 1 });
    }

    /// `uv` returns four 2D corner UVs in the `[0, 1]` range; corners
    /// span exactly the rect's footprint inside the atlas.
    #[test]
    fn uv_corners_cover_rect_footprint() {
        let mut atlas = GreedyAtlas::new(1024, 1024);
        let out = atlas.pack(&[rect(1, 256, 256)]).expect("single rect");
        let uvs = out[0].uv(0, 1024.0);
        // Top-left, top-right, bottom-right, bottom-left.
        let xs: Vec<f32> = uvs.iter().map(|v| v.x).collect();
        let ys: Vec<f32> = uvs.iter().map(|v| v.y).collect();
        assert_eq!(xs[0], 0.0);
        assert_eq!(xs[1], 0.25); // 256/1024
        assert!((xs[2] - 0.25).abs() < 1e-6);
        assert_eq!(xs[3], 0.0);
        assert_eq!(ys[0], 0.0);
        assert_eq!(ys[1], 0.0);
        assert_eq!(ys[2], 0.25);
        assert_eq!(ys[3], 0.25);
    }
}
