//! Greedy 2D atlas packer for the Civis PBR pipeline (FR-CIV-PBR-011).
//!
//! The Phase 3 PBR substrate builds a single RGBA `texture_2d_array` from
//! every material's albedo / normal / ORM maps. This module is the
//! CPU-side packing engine: it lays out rectangles into a power-of-two
//! atlas and emits the placement map (`HashMap<String, Rect>`) the
//! bevy adapter reads to bind each chunk's per-material UV.
//!
//! # Algorithm
//!
//! Shelf packing (a.k.a. "Next-Fit Decreasing Height" shelf packing):
//!
//! 1. Sort the input rectangles by `(height desc, width desc)`. Descending
//!    height gives a tighter packing than descending width on average.
//! 2. Walk the sorted list. For each rectangle, scan the existing shelves
//!    left-to-right; if the rectangle fits in a shelf (`cursor_x + w ≤
//!    atlas_w` and `shelf_height ≥ rect_height`), drop it on the first such
//!    shelf. Otherwise allocate a new shelf at `y = sum(shelf_heights_so_far)`
//!    when it fits within `atlas_h`, else fail.
//! 3. The shelf height is fixed at placement time — subsequent rectangles
//!    must be ≤ that height to share the shelf.
//!
//! Determinism: the sort key is total (`height` then `width` as tie-break)
//! and ids are unique per call, so two calls with the same input produce
//! byte-identical output. Replay-safe.
//!
//! # Public surface
//!
//! - [`GreedyAtlasPacker::new`] — construct a packer at the desired atlas size
//!   (defaults are exposed via [`DEFAULT_ATLAS_SIZE`]).
//! - [`GreedyAtlasPacker::pack`] — pack a list of input rectangles. Returns
//!   the assembled `atlas_texture` (RGBA bytes) and a placement map keyed by
//!   material name.
//! - [`GreedyAtlasPacker::pack_to_png`] — debug helper that writes a `.ppm`
//!   (Portable PixMap) next to the requested path. PPM is universally
//!   readable; the path is suffixed with `.ppm` to make the format obvious.
//!
//! # FR coverage
//!
//! - FR-CIV-PBR-011 — resolution-free material sheet; the Bevy adapter reads
//!   [`PackedAtlas::placements`] to bind chunk quads.
//! - This module is purely additive on top of FR-001..010; it does NOT
//!   touch any existing policy type.

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Default atlas edge length (texels). Power-of-two for GPU-friendly
/// mipmapping.
pub const DEFAULT_ATLAS_SIZE: u32 = 4096;

/// RGBA channels per atlas texel.
pub const CHANNELS_PER_TEXEL: usize = 4;

/// A rectangle of RGBA pixels in CPU memory — fed into
/// [`GreedyAtlasPacker::pack`] along with a stable material id (name).
///
/// `pixels` is laid out row-major with `width * CHANNELS_PER_TEXEL` bytes
/// per row and `height` rows. The packer copies bytes verbatim into the
/// atlas; no sRGB conversion, no premultiply. That is the caller's job
/// (see `material_pbr::ColorSpacePolicy`).
#[derive(Debug, Clone)]
pub struct InputTexture {
    /// Stable material id (used as the key in the returned placements map).
    pub name: String,
    /// Width in texels.
    pub width: u32,
    /// Height in texels.
    pub height: u32,
    /// RGBA bytes, `width * height * 4` long, row-major top-to-bottom.
    pub pixels: Vec<u8>,
}

impl InputTexture {
    /// Convenience constructor for a single-colour test rectangle. Useful
    /// for unit tests and debug fills.
    #[must_use]
    pub fn solid(name: impl Into<String>, width: u32, height: u32, rgba: [u8; 4]) -> Self {
        let count = (width as usize).saturating_mul(height as usize);
        let mut pixels = Vec::with_capacity(count * CHANNELS_PER_TEXEL);
        for _ in 0..count {
            pixels.extend_from_slice(&rgba);
        }
        Self {
            name: name.into(),
            width,
            height,
            pixels,
        }
    }

    /// Pixel count (`width * height`) — the number of texels this input
    /// contributes to the atlas.
    #[must_use]
    pub fn pixel_count(&self) -> usize {
        (self.width as usize) * (self.height as usize)
    }

    /// True when width/height are both positive and the pixel buffer
    /// length matches `width * height * 4`. Caught early so the packer
    /// never has to bounds-check byte arithmetic at hot path.
    #[must_use]
    pub fn is_well_formed(&self) -> bool {
        self.width > 0
            && self.height > 0
            && self.pixels.len() == self.pixel_count() * CHANNELS_PER_TEXEL
    }
}

/// A rectangle placement in the packed atlas. `x` and `y` are the
/// top-left corner in texel coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    /// Inclusive x0 of the placed rectangle (texels).
    pub x: u32,
    /// Inclusive y0 of the placed rectangle (texels).
    pub y: u32,
    /// Width in texels.
    pub width: u32,
    /// Height in texels.
    pub height: u32,
}

impl Rect {
    /// Inclusive right edge (x0 + width).
    #[must_use]
    pub fn right(&self) -> u32 {
        self.x + self.width
    }
    /// Inclusive bottom edge (y0 + height).
    #[must_use]
    pub fn bottom(&self) -> u32 {
        self.y + self.height
    }
}

/// The packer's output: the assembled RGBA atlas + the placement map.
#[derive(Debug, Clone)]
pub struct PackedAtlas {
    /// Width of the assembled atlas in texels (≤ packer's max).
    pub width: u32,
    /// Height of the assembled atlas in texels (≤ packer's max).
    pub height: u32,
    /// RGBA bytes, `width * height * 4` long, row-major top-to-bottom.
    /// Initialised to fully transparent black (RGBA = 0) so uncovered
    /// regions have a defined sample.
    pub atlas_texture: Vec<u8>,
    /// Placement map keyed by material name (the caller's `InputTexture::name`).
    pub placements: HashMap<String, Rect>,
}

/// Errors returned by [`GreedyAtlasPacker::pack`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AtlasError {
    /// Two input rectangles shared the same material name. The placement
    /// map is one-to-one by contract; reject loudly.
    DuplicateName(String),
    /// A rectangle was malformed (zero-sized or pixel buffer mismatch).
    MalformedInput { name: String, reason: &'static str },
    /// A rectangle's width or height exceeded the atlas dimension on
    /// either axis.
    RectTooLarge {
        name: String,
        width: u32,
        height: u32,
        atlas_width: u32,
        atlas_height: u32,
    },
    /// A rectangle could not fit on any existing shelf and would not fit
    /// on a new one either (atlas exhausted vertically).
    AtlasExhausted { name: String, atlas_height: u32 },
}

impl fmt::Display for AtlasError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateName(n) => write!(f, "duplicate atlas name: {n}"),
            Self::MalformedInput { name, reason } => {
                write!(f, "malformed input '{name}': {reason}")
            }
            Self::RectTooLarge {
                name,
                width,
                height,
                atlas_width,
                atlas_height,
            } => write!(
                f,
                "rect '{name}' ({width}x{height}) exceeds atlas {atlas_width}x{atlas_height}"
            ),
            Self::AtlasExhausted { name, atlas_height } => {
                write!(
                    f,
                    "atlas exhausted placing '{name}' (height {atlas_height})"
                )
            }
        }
    }
}

impl std::error::Error for AtlasError {}

/// Internal shelf representation. A shelf is a horizontal band whose
/// height is fixed at creation. Rectangles on a shelf must be ≤ shelf
/// height and are placed left-to-right by `cursor_x`.
#[derive(Debug, Clone)]
struct Shelf {
    /// Top-left y of the shelf.
    y: u32,
    /// Shelf height in texels.
    height: u32,
    /// Next-free x cursor (the right edge of the previously placed rect).
    cursor_x: u32,
}

/// Greedy 2D shelf atlas packer.
///
/// Construct with [`GreedyAtlasPacker::new`] (defaults to a 4096×4096 RGBA
/// atlas). Reusable across multiple `pack` calls; the atlas buffer is
/// rebuilt each time so callers do not need to clear state.
#[derive(Debug, Clone)]
pub struct GreedyAtlasPacker {
    max_width: u32,
    max_height: u32,
    /// Internal bookkeeping for the most-recent pack call.
    shelves: Vec<Shelf>,
}

impl GreedyAtlasPacker {
    /// Construct a packer at the given atlas dimensions.
    ///
    /// # Panics
    /// Panics if `max_width == 0`, `max_height == 0`, or either is not a
    /// power of two. Both constraints are loud failures: a zero-sized
    /// atlas cannot hold any rect, and a non-POT atlas breaks mipmapping
    /// downstream on most GPUs.
    #[must_use]
    pub fn new(max_width: u32, max_height: u32) -> Self {
        assert!(max_width > 0, "atlas max_width must be > 0");
        assert!(max_height > 0, "atlas max_height must be > 0");
        assert!(
            max_width.is_power_of_two(),
            "atlas max_width must be a power of two"
        );
        assert!(
            max_height.is_power_of_two(),
            "atlas max_height must be a power of two"
        );
        Self {
            max_width,
            max_height,
            shelves: Vec::new(),
        }
    }

    /// Maximum atlas width.
    #[must_use]
    pub fn max_width(&self) -> u32 {
        self.max_width
    }

    /// Maximum atlas height.
    #[must_use]
    pub fn max_height(&self) -> u32 {
        self.max_height
    }

    /// Number of shelves allocated by the most recent `pack` call.
    /// Diagnostics only; no behaviour depends on the count.
    #[must_use]
    pub fn shelf_count(&self) -> usize {
        self.shelves.len()
    }

    /// Pack a list of input rectangles into the atlas. The returned
    /// [`PackedAtlas`] is freshly allocated; callers may move or copy it.
    pub fn pack(&mut self, inputs: &[InputTexture]) -> Result<PackedAtlas, AtlasError> {
        self.shelves.clear();

        // Reject duplicates and malformed inputs up front so the packing
        // loop can stay total (no `?` on the hot path).
        let mut seen: HashMap<&str, ()> = HashMap::with_capacity(inputs.len());
        for input in inputs {
            if seen.insert(input.name.as_str(), ()).is_some() {
                return Err(AtlasError::DuplicateName(input.name.clone()));
            }
            if input.width == 0 || input.height == 0 {
                return Err(AtlasError::MalformedInput {
                    name: input.name.clone(),
                    reason: "zero-sized rect",
                });
            }
            if input.pixels.len() != input.pixel_count() * CHANNELS_PER_TEXEL {
                return Err(AtlasError::MalformedInput {
                    name: input.name.clone(),
                    reason: "pixel buffer length mismatch",
                });
            }
            if input.width > self.max_width || input.height > self.max_height {
                return Err(AtlasError::RectTooLarge {
                    name: input.name.clone(),
                    width: input.width,
                    height: input.height,
                    atlas_width: self.max_width,
                    atlas_height: self.max_height,
                });
            }
        }

        // Sort by descending height; ties broken by descending width.
        let mut order: Vec<usize> = (0..inputs.len()).collect();
        order.sort_by(|&a, &b| {
            inputs[b]
                .height
                .cmp(&inputs[a].height)
                .then_with(|| inputs[b].width.cmp(&inputs[a].width))
        });

        let mut placements: HashMap<String, Rect> = HashMap::with_capacity(inputs.len());

        for idx in order {
            let input = &inputs[idx];
            let w = input.width;
            let h = input.height;

            // First fit on an existing shelf.
            let mut chosen: Option<usize> = None;
            for (i, shelf) in self.shelves.iter().enumerate() {
                if shelf.height >= h && shelf.cursor_x + w <= self.max_width {
                    chosen = Some(i);
                    break;
                }
            }

            let rect = if let Some(i) = chosen {
                let shelf = &mut self.shelves[i];
                let rect = Rect {
                    x: shelf.cursor_x,
                    y: shelf.y,
                    width: w,
                    height: h,
                };
                shelf.cursor_x += w;
                rect
            } else {
                // Allocate a new shelf at the bottom of the used area.
                let shelf_bottom = self
                    .shelves
                    .iter()
                    .map(|s| s.y + s.height)
                    .max()
                    .unwrap_or(0);
                if shelf_bottom + h > self.max_height {
                    return Err(AtlasError::AtlasExhausted {
                        name: input.name.clone(),
                        atlas_height: self.max_height,
                    });
                }
                let rect = Rect {
                    x: 0,
                    y: shelf_bottom,
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

            placements.insert(input.name.clone(), rect);
        }

        // Assemble the atlas. Uncovered regions stay fully transparent
        // black so the GPU sampler returns a defined value.
        let mut atlas_texture =
            vec![0u8; (self.max_width as usize) * (self.max_height as usize) * CHANNELS_PER_TEXEL];

        for input in inputs {
            let rect = placements[&input.name];
            blit(&mut atlas_texture, self.max_width, input, rect);
        }

        Ok(PackedAtlas {
            width: self.max_width,
            height: self.max_height,
            atlas_texture,
            placements,
        })
    }

    /// Pack + write the assembled atlas as a binary PPM (Portable PixMap,
    /// P6) to `path` (suffix is rewritten to `.ppm`). PPM is a one-line
    /// header + raw bytes — universally readable by every image viewer
    /// and decodable with zero deps. This is the spec-mandated
    /// `pack_to_png` debug helper for FR-011 visual inspection; we use
    /// PPM under the hood to stay dep-free.
    pub fn pack_to_ppm(
        &mut self,
        inputs: &[InputTexture],
        path: impl AsRef<Path>,
    ) -> Result<PackedAtlas, AtlasError> {
        let packed = self.pack(inputs)?;
        let path = path.as_ref().with_extension("ppm");
        let file = File::create(&path)
            .unwrap_or_else(|e| panic!("atlas ppm: cannot create {}: {e}", path.display()));
        let mut writer = BufWriter::new(file);
        write_rgba_ppm(
            &mut writer,
            packed.width,
            packed.height,
            &packed.atlas_texture,
        )
        .unwrap_or_else(|e| panic!("atlas ppm: write failed: {e}"));
        writer.flush().expect("atlas ppm: flush failed");
        Ok(packed)
    }

    /// Alias for [`Self::pack_to_ppm`] kept under the spec name. Writes
    /// `.ppm` next to the requested path; an actual `.png` writer would
    /// pull in `image`/`png` crates which the substrate forbids.
    pub fn pack_to_png(
        &mut self,
        inputs: &[InputTexture],
        path: impl AsRef<Path>,
    ) -> Result<PackedAtlas, AtlasError> {
        self.pack_to_ppm(inputs, path)
    }
}

/// Copy a single input rectangle into the assembled atlas at `rect`.
/// Out-of-bounds writes are silently skipped — should never happen
/// because the packer validates rects before this call.
fn blit(atlas: &mut [u8], atlas_w: u32, input: &InputTexture, rect: Rect) {
    let row_bytes = (atlas_w as usize) * CHANNELS_PER_TEXEL;
    let rect_bottom = rect.y + rect.height;
    if rect.right() > atlas_w {
        return;
    }
    for row in 0..(rect.height as usize) {
        let dst_y = (rect.y as usize) + row;
        if dst_y >= rect_bottom as usize {
            break;
        }
        let dst_offset = dst_y * row_bytes + (rect.x as usize) * CHANNELS_PER_TEXEL;
        let src_offset = row * (input.width as usize) * CHANNELS_PER_TEXEL;
        let copy_len = (input.width as usize) * CHANNELS_PER_TEXEL;
        if dst_offset + copy_len > atlas.len() {
            break;
        }
        atlas[dst_offset..dst_offset + copy_len]
            .copy_from_slice(&input.pixels[src_offset..src_offset + copy_len]);
    }
}

/// Write a binary P6 PPM with the given RGBA pixels. The fourth (alpha)
/// channel is dropped because PPM is RGB-only; an alpha < 128 paints the
/// pixel as magenta for debug visibility (so empty regions of the atlas
/// are obvious in a viewer).
fn write_rgba_ppm<W: Write>(
    w: &mut W,
    width: u32,
    height: u32,
    rgba: &[u8],
) -> std::io::Result<()> {
    writeln!(w, "P6")?;
    writeln!(w, "{width} {height}")?;
    writeln!(w, "255")?;
    for chunk in rgba.chunks(CHANNELS_PER_TEXEL) {
        let alpha = chunk[3];
        if alpha < 128 {
            w.write_all(&[255, 0, 255])?;
        } else {
            w.write_all(&chunk[0..3])?;
        }
    }
    Ok(())
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// 1 texture: lands at (0, 0), exactly one shelf.
    #[test]
    fn pack_one_texture() {
        let mut packer = GreedyAtlasPacker::new(128, 128);
        let inputs = vec![InputTexture::solid("grass", 64, 32, [10, 20, 30, 255])];
        let packed = packer.pack(&inputs).expect("single rect packs");
        assert_eq!(packed.placements.len(), 1);
        let r = packed.placements["grass"];
        assert_eq!((r.x, r.y, r.width, r.height), (0, 0, 64, 32));
        assert_eq!(packed.atlas_texture.len(), 128 * 128 * CHANNELS_PER_TEXEL);
        assert_eq!(packer.shelf_count(), 1);
    }

    /// 5 textures of varied sizes: every placement is inside the atlas
    /// and no two rects overlap.
    #[test]
    fn pack_five_varied_textures() {
        let mut packer = GreedyAtlasPacker::new(512, 512);
        let inputs = vec![
            InputTexture::solid("a", 256, 128, [255, 0, 0, 255]),
            InputTexture::solid("b", 128, 256, [0, 255, 0, 255]),
            InputTexture::solid("c", 64, 64, [0, 0, 255, 255]),
            InputTexture::solid("d", 192, 96, [255, 255, 0, 255]),
            InputTexture::solid("e", 96, 192, [0, 255, 255, 255]),
        ];
        let packed = packer.pack(&inputs).expect("five varied pack");
        assert_eq!(packed.placements.len(), 5);

        let rects: Vec<(&str, Rect)> = packed
            .placements
            .iter()
            .map(|(k, v)| (k.as_str(), *v))
            .collect();
        for i in 0..rects.len() {
            for j in (i + 1)..rects.len() {
                let (a_name, a) = rects[i];
                let (b_name, b) = rects[j];
                let h_overlap = a.x < b.right() && b.x < a.right();
                let v_overlap = a.y < b.bottom() && b.y < a.bottom();
                assert!(
                    !(h_overlap && v_overlap),
                    "rects {a_name:?} and {b_name:?} overlap"
                );
            }
        }

        for (name, r) in &rects {
            assert!(r.right() <= 512, "{name} overflows width");
            assert!(r.bottom() <= 512, "{name} overflows height");
        }
    }

    /// 50 uniform 64×64 textures: pack successfully, no overlaps, total
    /// filled area equals `50 * 64 * 64`.
    #[test]
    fn pack_fifty_uniform_textures() {
        let mut packer = GreedyAtlasPacker::new(4096, 4096);
        let mut inputs: Vec<InputTexture> = Vec::with_capacity(50);
        for i in 0..50u32 {
            inputs.push(InputTexture::solid(
                format!("mat_{i}"),
                64,
                64,
                [
                    (i % 256) as u8,
                    ((i * 3) % 256) as u8,
                    ((i * 7) % 256) as u8,
                    255,
                ],
            ));
        }
        let packed = packer.pack(&inputs).expect("50 uniform pack");
        assert_eq!(packed.placements.len(), 50);

        let total_filled: u64 = packed
            .placements
            .values()
            .map(|r| (r.width as u64) * (r.height as u64))
            .sum();
        let expected: u64 = 50 * 64 * 64;
        assert_eq!(total_filled, expected);

        let rects: Vec<Rect> = packed.placements.values().copied().collect();
        for i in 0..rects.len() {
            for j in (i + 1)..rects.len() {
                let a = rects[i];
                let b = rects[j];
                let h_overlap = a.x < b.right() && b.x < a.right();
                let v_overlap = a.y < b.bottom() && b.y < a.bottom();
                assert!(!(h_overlap && v_overlap), "rect {i} and {j} overlap");
            }
        }
    }

    /// Duplicate names are rejected with `DuplicateName`.
    #[test]
    fn pack_rejects_duplicate_names() {
        let mut packer = GreedyAtlasPacker::new(256, 256);
        let inputs = vec![
            InputTexture::solid("dup", 32, 32, [255; 4]),
            InputTexture::solid("dup", 32, 32, [0; 4]),
        ];
        let err = packer.pack(&inputs).expect_err("dup rejected");
        assert_eq!(err, AtlasError::DuplicateName("dup".to_string()));
    }

    /// Malformed inputs (zero-sized, pixel mismatch) are rejected.
    #[test]
    fn pack_rejects_malformed_inputs() {
        let mut packer = GreedyAtlasPacker::new(128, 128);

        let mut zero = InputTexture::solid("z", 1, 1, [0; 4]);
        zero.width = 0;
        zero.height = 0;
        zero.pixels.clear();
        assert!(matches!(
            packer.pack(&[zero]),
            Err(AtlasError::MalformedInput {
                name,
                reason: "zero-sized rect",
            }) if name == "z"
        ));

        let mut bad = InputTexture::solid("b", 4, 4, [1, 2, 3, 4]);
        bad.pixels.truncate(3);
        assert!(matches!(
            packer.pack(&[bad]),
            Err(AtlasError::MalformedInput { .. })
        ));
    }

    /// A rectangle bigger than the atlas on either axis is rejected with
    /// `RectTooLarge`.
    #[test]
    fn pack_rejects_oversized_rect() {
        let mut packer = GreedyAtlasPacker::new(64, 64);
        let inputs = vec![InputTexture::solid("big", 128, 32, [0; 4])];
        let err = packer.pack(&inputs).expect_err("oversized rejected");
        assert!(matches!(err, AtlasError::RectTooLarge { .. }));
    }

    /// Many small rects that exhaust the vertical space fail with
    /// `AtlasExhausted` rather than panicking.
    #[test]
    fn pack_reports_atlas_exhaustion() {
        let mut packer = GreedyAtlasPacker::new(32, 32);
        let inputs: Vec<InputTexture> = (0..33)
            .map(|i| InputTexture::solid(format!("r{i}"), 32, 32, [0; 4]))
            .collect();
        let err = packer.pack(&inputs).expect_err("atlas exhausted");
        assert!(matches!(err, AtlasError::AtlasExhausted { .. }));
    }

    /// `is_well_formed` returns true for correctly sized inputs.
    #[test]
    fn input_texture_well_formed_predicate() {
        let good = InputTexture::solid("g", 4, 4, [1, 2, 3, 4]);
        assert!(good.is_well_formed());

        let mut bad = InputTexture::solid("b", 4, 4, [0; 4]);
        bad.pixels.clear();
        assert!(!bad.is_well_formed());
    }

    /// `GreedyAtlasPacker::new` rejects zero / non-power-of-two dims.
    #[test]
    #[should_panic]
    fn packer_new_rejects_zero_width() {
        let _ = GreedyAtlasPacker::new(0, 64);
    }

    #[test]
    #[should_panic]
    fn packer_new_rejects_non_power_of_two() {
        let _ = GreedyAtlasPacker::new(100, 64);
    }

    /// Default atlas size constant is 4096 and a power of two.
    #[test]
    fn default_atlas_size_is_pow2() {
        assert_eq!(DEFAULT_ATLAS_SIZE, 4096);
        assert!(DEFAULT_ATLAS_SIZE.is_power_of_two());
    }

    /// Empty input list packs into a fully transparent atlas.
    #[test]
    fn pack_empty_input_yields_transparent_atlas() {
        let mut packer = GreedyAtlasPacker::new(64, 64);
        let packed = packer.pack(&[]).expect("empty pack");
        assert!(packed.placements.is_empty());
        assert_eq!(packed.atlas_texture.len(), 64 * 64 * CHANNELS_PER_TEXEL);
        assert!(packed.atlas_texture.iter().all(|&b| b == 0));
    }
}
