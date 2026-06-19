//! Shared live-scene focus bounds for orbit camera and HUD minimap framing.

use bevy::prelude::*;
use civ_voxel::ChunkId;

use crate::live_minimap::chunk_centre_world_xz;
use crate::live_stream::{
    LiveAgentTag, LiveBuildingTag, LiveGraphParcelTag, LiveStreamScene, LIVE_CHUNK_EDGE,
};
use crate::terrain::WORLD_SIZE;

/// Orbit / minimap centre lerp speed when following streamed entity bounds.
pub const LIVE_FOCUS_LERP_SPEED: f32 = 2.5;

/// Minimum orthographic half-extent (world units) so the minimap does not over-zoom.
pub const LIVE_FOCUS_MIN_HALF_EXTENT: f32 = 32.0;

/// Smoothed world-space centre and half-extent for live attach camera + minimap framing.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct LiveSceneFocus {
    /// World-space centre (XZ from streamed entities).
    pub centre: Vec3,
    /// Half-width of the minimap / focus view in world units.
    pub half_extent: f32,
}

impl Default for LiveSceneFocus {
    fn default() -> Self {
        Self {
            centre: Vec3::ZERO,
            half_extent: WORLD_SIZE * 0.5,
        }
    }
}

impl LiveSceneFocus {
    /// Map world XZ into normalised minimap UV (UI top-left origin).
    #[must_use]
    pub fn world_to_minimap_uv(&self, x: f32, z: f32) -> [f32; 2] {
        let uv = world_to_minimap_uv_focus(Vec3::new(x, 0.0, z), *self);
        [uv.x, uv.y]
    }
}

/// Compute focus bounds from streamed chunks, agents, buildings, and graph parcels.
#[must_use]
pub fn compute_live_scene_focus(
    scene: &LiveStreamScene,
    agents: &Query<&Transform, With<LiveAgentTag>>,
    buildings: &Query<&Transform, With<LiveBuildingTag>>,
    graph_parcels: &Query<&Transform, With<LiveGraphParcelTag>>,
) -> LiveSceneFocus {
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_z = f32::MAX;
    let mut max_z = f32::MIN;

    let mut extend = |x: f32, z: f32| {
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_z = min_z.min(z);
        max_z = max_z.max(z);
    };

    for raw in scene.chunks.keys() {
        let (x, z) = chunk_centre_world_xz(ChunkId(*raw), LIVE_CHUNK_EDGE);
        extend(x, z);
    }
    for transform in agents.iter() {
        extend(transform.translation.x, transform.translation.z);
    }
    for transform in buildings.iter() {
        extend(transform.translation.x, transform.translation.z);
    }
    for transform in graph_parcels.iter() {
        extend(transform.translation.x, transform.translation.z);
    }

    if min_x == f32::MAX {
        return LiveSceneFocus::default();
    }

    let centre = Vec3::new((min_x + max_x) * 0.5, 0.0, (min_z + max_z) * 0.5);
    let half_extent = ((max_x - min_x).max(max_z - min_z) * 0.55)
        .max(LIVE_FOCUS_MIN_HALF_EXTENT)
        .min(WORLD_SIZE * 0.5);
    LiveSceneFocus {
        centre,
        half_extent,
    }
}

/// Map world XZ into normalised minimap UV within `focus` bounds (`v` flipped for UI top-left).
#[must_use]
pub fn world_to_minimap_uv_focus(position: Vec3, focus: LiveSceneFocus) -> Vec2 {
    let min_x = focus.centre.x - focus.half_extent;
    let max_x = focus.centre.x + focus.half_extent;
    let min_z = focus.centre.z - focus.half_extent;
    let max_z = focus.centre.z + focus.half_extent;
    let span_x = (max_x - min_x).max(f32::EPSILON);
    let span_z = (max_z - min_z).max(f32::EPSILON);
    let u = ((position.x - min_x) / span_x).clamp(0.0, 1.0);
    let v = ((position.z - min_z) / span_z).clamp(0.0, 1.0);
    Vec2::new(u, 1.0 - v)
}

/// Inverse of [`world_to_minimap_uv_focus`]: minimap UV → world XZ.
#[must_use]
pub fn minimap_uv_to_world_xz(uv: Vec2, focus: LiveSceneFocus) -> (f32, f32) {
    let min_x = focus.centre.x - focus.half_extent;
    let min_z = focus.centre.z - focus.half_extent;
    let span = (focus.half_extent * 2.0).max(f32::EPSILON);
    let x = min_x + uv.x.clamp(0.0, 1.0) * span;
    let z = min_z + (1.0 - uv.y.clamp(0.0, 1.0)) * span;
    (x, z)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-CIV-BEVY-016 — live focus round-trips world-space to minimap UV and back.
    /// FR-CIV-BEVY-022 — focus-driven minimap UV mapping used by shared live scene tooling.
    #[test]
    fn minimap_uv_roundtrips_world_xz() {
        let focus = LiveSceneFocus {
            centre: Vec3::new(64.0, 0.0, 64.0),
            half_extent: 48.0,
        };
        let world = Vec3::new(80.0, 0.0, 40.0);
        let uv = world_to_minimap_uv_focus(world, focus);
        let (x, z) = minimap_uv_to_world_xz(uv, focus);
        assert!((x - world.x).abs() < 0.01);
        assert!((z - world.z).abs() < 0.01);
    }
}
