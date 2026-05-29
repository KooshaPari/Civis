//! Shared live-attach minimap dot layout, UV mapping, and spawn helpers.
//!
//! Used by [`crate::live_scene`] (render-to-texture minimap) and
//! [`crate::bin::bevy_window`] (inset HUD minimap).

use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use civ_voxel::ChunkId;

use crate::minimap::MinimapDot;
use crate::{chunk_to_minimap_uv, decode_chunk_id, world_xz_to_minimap_uv, MinimapBounds};
use civ_protocol_3d::BuildingProvenance;

/// Re-export building provenance tint from [`live_stream`](crate::live_stream).
pub use crate::live_stream::building_minimap_dot_color;

/// Default live-stream entity dot diameter in UI pixels.
pub const LIVE_MINIMAP_DOT: f32 = 4.0;

/// Slightly smaller dot for graph parcels on the minimap.
pub const LIVE_MINIMAP_GRAPH_DOT_SCALE: f32 = 0.85;

/// Agent marker tint on the live minimap.
pub const LIVE_MINIMAP_AGENT_COLOR: Color = Color::srgba(0.35, 0.82, 0.95, 1.0);

/// Chunk marker tint on the live minimap (unfocused).
pub const LIVE_MINIMAP_CHUNK_COLOR: Color = Color::srgba(0.55, 0.58, 0.62, 0.9);

/// Chunk marker tint when the HUD marks that chunk as focused (`bevy_window`).
pub const LIVE_MINIMAP_CHUNK_FOCUSED_COLOR: Color = Color::srgb(0.95, 0.92, 0.45);

/// Chunk marker tint when loaded but not focused (`bevy_window`).
pub const LIVE_MINIMAP_CHUNK_LOADED_COLOR: Color = Color::srgb(0.72, 0.69, 0.62);

/// Camera / orbit centre marker on the HUD minimap.
pub const LIVE_MINIMAP_CAMERA_COLOR: Color = Color::srgb(0.95, 0.95, 0.98);

/// World-space rectangle for focus-driven minimap UV (live_scene ortho camera).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinimapFocusRect {
    /// World X centre.
    pub centre_x: f32,
    /// World Z centre.
    pub centre_z: f32,
    /// Half-width of the visible XZ region in world units.
    pub half_extent: f32,
}

impl MinimapFocusRect {
    /// Maps world XZ into normalised minimap UV with V flipped for UI top-left origin.
    #[must_use]
    pub fn world_to_uv(&self, x: f32, z: f32) -> [f32; 2] {
        let min_x = self.centre_x - self.half_extent;
        let max_x = self.centre_x + self.half_extent;
        let min_z = self.centre_z - self.half_extent;
        let max_z = self.centre_z + self.half_extent;
        let span_x = (max_x - min_x).max(f32::EPSILON);
        let span_z = (max_z - min_z).max(f32::EPSILON);
        let u = ((x - min_x) / span_x).clamp(0.0, 1.0);
        let v = ((z - min_z) / span_z).clamp(0.0, 1.0);
        [u, 1.0 - v]
    }
}

/// How normalised minimap UV maps into a UI panel for dot placement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MinimapDotLayout {
    /// UV spans the full square panel (live_scene render-to-texture minimap).
    FullPanel {
        /// Panel side length in pixels.
        panel_size: f32,
    },
    /// UV spans an inset plot inside the panel (`bevy_window` HUD).
    InsetHud {
        /// Panel side length in pixels.
        panel_size: f32,
        /// Padding from panel edge to plot.
        inset: f32,
        /// Margin subtracted from plot width (matches reference dot size).
        plot_margin_dot: f32,
    },
}

impl MinimapDotLayout {
    /// Top-left `(left, top)` in pixels for a circular dot centred at `uv`.
    #[must_use]
    pub fn dot_origin(&self, uv: [f32; 2], dot_size: f32) -> (f32, f32) {
        let (plot_origin, plot_size) = match *self {
            Self::FullPanel { panel_size } => (0.0, panel_size),
            Self::InsetHud {
                panel_size,
                inset,
                plot_margin_dot,
            } => (inset, panel_size - inset * 2.0 - plot_margin_dot),
        };
        let left = plot_origin + uv[0] * plot_size - dot_size * 0.5;
        let top = plot_origin + uv[1] * plot_size - dot_size * 0.5;
        (left, top)
    }
}

/// Chunk-grid bounds from loaded chunk keys, or `None` when empty.
#[must_use]
pub fn minimap_bounds_from_keys(chunk_keys: &[u64]) -> Option<MinimapBounds> {
    let mut min_x = i32::MAX;
    let mut min_z = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_z = i32::MIN;
    for &raw in chunk_keys {
        let (cx, _cy, cz) = decode_chunk_id(ChunkId(raw));
        min_x = min_x.min(cx);
        min_z = min_z.min(cz);
        max_x = max_x.max(cx);
        max_z = max_z.max(cz);
    }
    if min_x == i32::MAX {
        None
    } else {
        Some((min_x, min_z, max_x, max_z))
    }
}

/// World XZ at the centre of a chunk cell (Y omitted).
#[must_use]
pub fn chunk_centre_world_xz(chunk_id: ChunkId, chunk_edge: usize) -> (f32, f32) {
    let (cx, _cy, cz) = decode_chunk_id(chunk_id);
    let edge = chunk_edge as f32;
    ((cx as f32 + 0.5) * edge, (cz as f32 + 0.5) * edge)
}

/// UV for a chunk centre within chunk-grid `bounds`.
#[must_use]
pub fn chunk_centre_minimap_uv(chunk_id: ChunkId, bounds: MinimapBounds) -> [f32; 2] {
    chunk_to_minimap_uv(chunk_id, bounds)
}

/// UV for world XZ within chunk-grid `bounds`.
#[must_use]
pub fn world_minimap_uv(x: f32, z: f32, bounds: MinimapBounds) -> [f32; 2] {
    world_xz_to_minimap_uv(x, z, bounds)
}

/// Building dot color from provenance (thin wrapper for call-site clarity).
#[must_use]
pub fn live_building_dot_color(provenance: BuildingProvenance) -> Color {
    building_minimap_dot_color(provenance)
}

/// Spawns a circular minimap dot under `parent` using `layout` and optional [`MinimapDot`] tag.
pub fn spawn_minimap_dot(
    parent: &mut ChildSpawnerCommands,
    layout: MinimapDotLayout,
    uv: [f32; 2],
    size: f32,
    color: Color,
    tag_minimap_dot: bool,
) {
    let (left, top) = layout.dot_origin(uv, size);
    let mut entity = parent.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(left),
            top: Val::Px(top),
            width: Val::Px(size),
            height: Val::Px(size),
            border_radius: BorderRadius::MAX,
            ..default()
        },
        BackgroundColor(color),
        FocusPolicy::Pass,
    ));
    if tag_minimap_dot {
        entity.insert(MinimapDot);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_rect_maps_centre_to_mid_uv() {
        let focus = MinimapFocusRect {
            centre_x: 100.0,
            centre_z: 200.0,
            half_extent: 50.0,
        };
        let uv = focus.world_to_uv(100.0, 200.0);
        assert!((uv[0] - 0.5).abs() < 1e-5);
        assert!((uv[1] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn focus_rect_flips_v_for_ui_origin() {
        let focus = MinimapFocusRect {
            centre_x: 0.0,
            centre_z: 0.0,
            half_extent: 10.0,
        };
        let top = focus.world_to_uv(0.0, 10.0);
        let bottom = focus.world_to_uv(0.0, -10.0);
        assert!(top[1] < bottom[1]);
    }

    #[test]
    fn inset_layout_offsets_by_inset() {
        let layout = MinimapDotLayout::InsetHud {
            panel_size: 160.0,
            inset: 6.0,
            plot_margin_dot: 4.0,
        };
        let (left, top) = layout.dot_origin([0.0, 0.0], 4.0);
        assert!((left - 4.0).abs() < 1e-5);
        assert!((top - 4.0).abs() < 1e-5);
    }

    #[test]
    fn bounds_from_keys_empty_is_none() {
        assert!(minimap_bounds_from_keys(&[]).is_none());
    }

    #[test]
    fn bounds_from_keys_spans_chunks() {
        let raw = ChunkId(0).0;
        let bounds = minimap_bounds_from_keys(&[raw]).expect("bounds");
        assert_eq!(bounds, (0, 0, 0, 0));
    }
}
