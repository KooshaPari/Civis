//! Engine-agnostic chunk LOD helpers for the future renderer pass.
//!
//! The later Bevy renderer will use this module as a very small planning shim:
//! frustum-cull chunk bounds first, compute the chunk distance from the camera,
//! then request a mesh detail level and feed the selected chunk into a
//! chunked-greedy mesher. Keeping this here avoids leaking renderer types into
//! the voxel substrate.

use crate::{select_lod, ChunkId, LodLevel, LodPolicy, VoxelScaleMultiplier};

/// Render planning output for a visible chunk.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChunkRenderPlan {
    /// Chunk identifier.
    pub chunk_id: ChunkId,
    /// Selected mesh detail level.
    pub lod: LodLevel,
    /// Distance in world metres from the camera to the chunk center.
    pub distance_metres: f32,
}

/// Select the mesh detail level for a chunk at the given distance.
#[must_use]
pub fn select_mesh_detail_level(
    distance_metres: f32,
    scale: VoxelScaleMultiplier,
    policy: LodPolicy,
) -> LodLevel {
    select_lod(distance_metres, scale, policy)
}

/// Build a render plan for the renderer after it has frustum-culled a chunk.
#[must_use]
pub fn plan_chunk_render(
    chunk_id: ChunkId,
    distance_metres: f32,
    in_frustum: bool,
    scale: VoxelScaleMultiplier,
    policy: LodPolicy,
) -> Option<ChunkRenderPlan> {
    if !in_frustum {
        return None;
    }
    Some(ChunkRenderPlan {
        chunk_id,
        lod: select_mesh_detail_level(distance_metres, scale, policy),
        distance_metres,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_selection_tracks_scale_invariance() {
        let policy = LodPolicy::default();
        let lod_a = select_mesh_detail_level(64.0 * 8.0, VoxelScaleMultiplier(8.0), policy);
        let lod_b = select_mesh_detail_level(64.0 * 16.0, VoxelScaleMultiplier(16.0), policy);
        assert_eq!(lod_a, lod_b);
    }

    #[test]
    fn plan_is_culled_before_lod_selection() {
        let policy = LodPolicy::default();
        assert!(
            plan_chunk_render(ChunkId(3), 32.0, false, VoxelScaleMultiplier::default(), policy)
                .is_none()
        );
        let plan = plan_chunk_render(
            ChunkId(7),
            1.0e6,
            true,
            VoxelScaleMultiplier::default(),
            policy,
        )
        .expect("visible chunk");
        assert_eq!(plan.chunk_id, ChunkId(7));
        assert_eq!(plan.lod, LodLevel(policy.max_level));
    }
}
