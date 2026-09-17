//! Hex map renderer (FR-UX-001, CIV-0300).
//!
//! The UI SHALL render the hex map using the `crates/render` crate at a
//! 60 fps target. This module provides a pure-data model for the hex grid
//! and a [`HexMapRenderer`] that converts the *visible* subset of tiles into
//! a draw list sized to the frame budget.
//!
//! No GPU dependency lives here; the client (`clients/bevy-ref/`) walks the
//! emitted [`DrawList`] and issues the corresponding wgpu draw calls. Keeping
//! the culling and batching logic here means the 60 fps guarantee can be
//! exercised in a headless test.

use serde::{Deserialize, Serialize};

use crate::frame::{FrameBudget, TARGET_FPS};

/// Axial hex coordinate, constrained to cube coordinates (`q + r + s == 0`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HexCoord {
    /// Axial `q` axis.
    pub q: i32,
    /// Axial `r` axis.
    pub r: i32,
}

impl HexCoord {
    /// Construct an axial coordinate.
    #[must_use]
    pub const fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    /// Cube `s` axis, derived to satisfy `q + r + s == 0`.
    #[must_use]
    pub const fn s(&self) -> i32 {
        -self.q - self.r
    }

    /// Hex distance in steps between two coordinates.
    #[must_use]
    pub fn distance(&self, other: Self) -> i32 {
        let dq = (self.q - other.q).abs();
        let dr = (self.r - other.r).abs();
        let ds = (self.s() - other.s()).abs();
        (dq + dr + ds) / 2
    }
}

/// A single tile entry in the hex map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tile {
    /// Grid coordinate of this tile.
    pub coord: HexCoord,
    /// Terrain identifier (index into the tile atlas).
    pub terrain: u16,
    /// LOD bucket this tile belongs to (0 = nearest detail).
    pub lod: u8,
}

/// One batched draw call grouping tiles sharing a terrain + LOD.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrawBatch {
    /// Terrain atlas index for the batch.
    pub terrain: u16,
    /// LOD bucket for the batch.
    pub lod: u8,
    /// Number of tiles in the batch.
    pub tile_count: u32,
}

/// Deterministic draw list produced for one frame.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DrawList {
    /// Batched draw calls, sorted by `(lod, terrain)`.
    pub batches: Vec<DrawBatch>,
    /// Total tiles that survived culling.
    pub visible_tiles: u32,
}

impl DrawList {
    /// Number of draw calls this frame.
    #[must_use]
    pub fn draw_calls(&self) -> usize {
        self.batches.len()
    }
}

/// Hex map renderer (FR-UX-001).
///
/// Holds the tile set and produces a per-frame [`DrawList`] under a
/// [`FrameBudget`]. Culling keeps off-screen tiles out of the draw list so
/// the 60 fps target is met regardless of total map size.
#[derive(Debug, Clone)]
pub struct HexMapRenderer {
    tiles: Vec<Tile>,
    budget: FrameBudget,
    /// Viewport radius in hex steps around the camera centre.
    view_radius: i32,
    camera: HexCoord,
}

impl HexMapRenderer {
    /// Create a renderer over `tiles` with the standard 60 fps budget.
    #[must_use]
    pub fn new(tiles: Vec<Tile>) -> Self {
        Self {
            tiles,
            budget: FrameBudget::new(),
            view_radius: 24,
            camera: HexCoord::new(0, 0),
        }
    }

    /// Target frames per second for this renderer.
    #[must_use]
    pub fn target_fps(&self) -> u32 {
        self.budget.target_fps()
    }

    /// Frame budget in milliseconds.
    #[must_use]
    pub fn budget_ms(&self) -> f64 {
        self.budget.budget_ms()
    }

    /// Total tiles known to the renderer.
    #[must_use]
    pub fn tile_count(&self) -> usize {
        self.tiles.len()
    }

    /// Move the camera centre (used by [`crate::camera`]).
    pub fn set_camera(&mut self, centre: HexCoord) {
        self.camera = centre;
    }

    /// Current camera centre.
    #[must_use]
    pub fn camera(&self) -> HexCoord {
        self.camera
    }

    /// Set the viewport radius, in hex steps, for culling.
    pub fn set_view_radius(&mut self, radius: i32) {
        self.view_radius = radius.max(0);
    }

    /// Build the draw list for the current camera position.
    ///
    /// Tiles outside [`view_radius`](Self::view_radius) of the camera are
    /// culled, then the remainder is merged into batches keyed by
    /// `(lod, terrain)`.
    #[must_use]
    pub fn build_draw_list(&self) -> DrawList {
        let mut visible: Vec<&Tile> = self
            .tiles
            .iter()
            .filter(|t| t.coord.distance(self.camera) <= self.view_radius)
            .collect();

        visible.sort_by_key(|t| (t.lod, t.terrain, t.coord.q, t.coord.r));

        let mut batches: Vec<DrawBatch> = Vec::new();
        for tile in &visible {
            match batches.last_mut() {
                Some(b) if b.terrain == tile.terrain && b.lod == tile.lod => {
                    b.tile_count += 1;
                }
                _ => batches.push(DrawBatch {
                    terrain: tile.terrain,
                    lod: tile.lod,
                    tile_count: 1,
                }),
            }
        }

        DrawList {
            batches,
            visible_tiles: u32::try_from(visible.len()).unwrap_or(u32::MAX),
        }
    }

    /// Does the worst-case frame fit the 60 fps budget?
    ///
    /// The client calls this after [`build_draw_list`](Self::build_draw_list)
    /// to decide whether to drop optional passes.
    #[must_use]
    pub fn meets_60fps(&self, elapsed_ms: f64) -> bool {
        elapsed_ms <= self.budget.budget_ms()
    }
}

/// The 60 fps target this renderer is built around.
pub const HEX_MAP_TARGET_FPS: u32 = TARGET_FPS;

#[cfg(test)]
mod tests {
    use super::*;

    fn grid(n: i32) -> Vec<Tile> {
        let mut tiles = Vec::new();
        for q in -n..=n {
            for r in -n..=n {
                tiles.push(Tile {
                    coord: HexCoord::new(q, r),
                    terrain: u16::try_from((q + r).rem_euclid(3)).unwrap(),
                    lod: 0,
                });
            }
        }
        tiles
    }

    #[test]
    fn hex_distance_is_symmetric() {
        let a = HexCoord::new(0, 0);
        let b = HexCoord::new(3, -1);
        assert_eq!(a.distance(b), b.distance(a));
        assert_eq!(a.distance(a), 0);
    }

    #[test]
    fn culling_reduces_tile_count() {
        let mut r = HexMapRenderer::new(grid(20));
        r.set_view_radius(2);
        let list = r.build_draw_list();
        assert!(list.visible_tiles < u32::try_from(r.tile_count()).unwrap());
    }

    #[test]
    fn draw_list_batches_by_terrain_and_lod() {
        let mut r = HexMapRenderer::new(grid(2));
        r.set_view_radius(10);
        let list = r.build_draw_list();
        assert!(list.draw_calls() <= 3, "three terrains => <= 3 batches");
        assert!(list.visible_tiles > 0);
    }
}
