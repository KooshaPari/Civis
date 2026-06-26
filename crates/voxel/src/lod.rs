//! Engine-agnostic chunk LOD helpers for the future renderer pass.
//!
//! The later Bevy renderer will use this module as a very small planning shim:
//! frustum-cull chunk bounds first, compute the chunk distance from the camera,
//! then request a mesh detail level and feed the selected chunk into a
//! chunked-greedy mesher. Keeping this here avoids leaking renderer types into
//! the voxel substrate.

use std::collections::HashSet;

use glam::IVec3;

use crate::{select_lod, ChunkId, LodLevel, LodPolicy, MeshBuffer, VoxelScaleMultiplier};

/// Why a chunk needs to be reprocessed by the remesh pipeline.
///
/// `StorageChanged` means the voxel source data changed and the pipeline must
/// refresh the snapshot from storage before meshing.
/// `MeshLodChanged` means the stored voxel snapshot is still valid, but the
/// chunk needs a new tessellation pass for the current mesh LOD.
/// `Both` means the chunk needs the storage refresh path and also changed LOD;
/// it follows the storage-refresh path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChunkDirty {
    StorageChanged,
    MeshLodChanged,
    Both,
}

impl ChunkDirty {
    /// True when the remesh pipeline must re-read the voxel snapshot before meshing.
    #[must_use]
    pub const fn requires_storage_refresh(self) -> bool {
        matches!(self, Self::StorageChanged | Self::Both)
    }
}

fn merge_chunk_dirty(existing: Option<ChunkDirty>, incoming: ChunkDirty) -> ChunkDirty {
    match (existing, incoming) {
        (None, dirty) => dirty,
        (Some(ChunkDirty::Both), _) | (_, ChunkDirty::Both) => ChunkDirty::Both,
        (Some(ChunkDirty::StorageChanged), ChunkDirty::MeshLodChanged)
        | (Some(ChunkDirty::MeshLodChanged), ChunkDirty::StorageChanged) => ChunkDirty::Both,
        (Some(current), incoming) if current == incoming => current,
        (_, incoming) => incoming,
    }
}

fn update_chunk_dirty(dirty: &mut HashSet<(IVec3, ChunkDirty)>, pos: IVec3, incoming: ChunkDirty) {
    let existing = dirty
        .iter()
        .find_map(|(chunk_pos, chunk_dirty)| (*chunk_pos == pos).then_some(*chunk_dirty));
    dirty.retain(|(chunk_pos, _)| *chunk_pos != pos);
    dirty.insert((pos, merge_chunk_dirty(existing, incoming)));
}

/// Mark a chunk as storage-dirty.
///
/// This is the stronger signal. If the chunk already had a mesh-LOD-only entry,
/// the entry is promoted to `ChunkDirty::Both` so the remesh pipeline takes the
/// full storage refresh path.
pub fn mark_storage_dirty(dirty: &mut HashSet<(IVec3, ChunkDirty)>, pos: IVec3) {
    update_chunk_dirty(dirty, pos, ChunkDirty::StorageChanged);
}

/// Mark a chunk as mesh-LOD-dirty.
///
/// If the chunk is already storage-dirty, the combined entry becomes
/// `ChunkDirty::Both` so consumers can still see that the chunk also needs a
/// retessellation at the current LOD.
pub fn mark_lod_dirty(dirty: &mut HashSet<(IVec3, ChunkDirty)>, pos: IVec3) {
    update_chunk_dirty(dirty, pos, ChunkDirty::MeshLodChanged);
}

/// Drain dirty chunk entries in a deterministic order.
///
/// The underlying storage is a `HashSet` for deduplication, so this helper is the
/// stable handoff point for remesh work queues.
pub fn drain_dirty_chunks(dirty: &mut HashSet<(IVec3, ChunkDirty)>) -> Vec<(IVec3, ChunkDirty)> {
    let mut out: Vec<_> = dirty.drain().collect();
    out.sort_unstable_by(|(left_pos, left_kind), (right_pos, right_kind)| {
        left_pos
            .x
            .cmp(&right_pos.x)
            .then(left_pos.y.cmp(&right_pos.y))
            .then(left_pos.z.cmp(&right_pos.z))
            .then_with(|| dirty_priority(*left_kind).cmp(&dirty_priority(*right_kind)))
    });
    out
}

const fn dirty_priority(kind: ChunkDirty) -> u8 {
    match kind {
        ChunkDirty::MeshLodChanged => 0,
        ChunkDirty::StorageChanged | ChunkDirty::Both => 1,
    }
}

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

/// Return the number of triangles encoded by a mesh buffer.
///
/// `CubicMesher` emits indexed triangle lists, so the index count is the stable
/// proxy for face complexity and triangle count is `indices.len() / 3`.
#[must_use]
pub fn mesh_triangle_count(mesh: &MeshBuffer) -> usize {
    mesh.indices.len() / 3
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
    use glam::IVec3;
    use std::collections::HashSet;

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
        assert!(plan_chunk_render(
            ChunkId(3),
            32.0,
            false,
            VoxelScaleMultiplier::default(),
            policy
        )
        .is_none());
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

    #[test]
    fn storage_dirty_promotes_lod_dirty_to_both() {
        let mut dirty = HashSet::new();
        let pos = IVec3::new(4, -2, 9);

        mark_lod_dirty(&mut dirty, pos);
        mark_storage_dirty(&mut dirty, pos);

        assert_eq!(dirty.len(), 1);
        assert!(dirty.contains(&(pos, ChunkDirty::Both)));
    }

    #[test]
    fn lod_dirty_is_deduplicated_per_chunk() {
        let mut dirty = HashSet::new();
        let pos = IVec3::new(1, 2, 3);

        mark_lod_dirty(&mut dirty, pos);
        mark_lod_dirty(&mut dirty, pos);
        mark_lod_dirty(&mut dirty, pos);

        assert_eq!(dirty.len(), 1);
        assert!(dirty.contains(&(pos, ChunkDirty::MeshLodChanged)));
    }

    #[test]
    fn drain_dirty_chunks_is_deterministic_and_prefers_storage_path() {
        let mut dirty = HashSet::new();
        let a = IVec3::new(2, 0, 0);
        let b = IVec3::new(1, 0, 0);
        let c = IVec3::new(3, 0, 0);

        mark_lod_dirty(&mut dirty, a);
        mark_storage_dirty(&mut dirty, b);
        mark_lod_dirty(&mut dirty, c);
        mark_storage_dirty(&mut dirty, c);

        let drained = drain_dirty_chunks(&mut dirty);
        assert!(dirty.is_empty());
        assert_eq!(
            drained,
            vec![
                (IVec3::new(1, 0, 0), ChunkDirty::StorageChanged),
                (IVec3::new(2, 0, 0), ChunkDirty::MeshLodChanged),
                (IVec3::new(3, 0, 0), ChunkDirty::Both),
            ]
        );
        assert!(drained[0].1.requires_storage_refresh());
        assert!(drained[2].1.requires_storage_refresh());
        assert!(!drained[1].1.requires_storage_refresh());
    }

    #[test]
    fn mesh_triangle_count_uses_index_triplets() {
        let mesh = MeshBuffer {
            vertices: vec![],
            indices: vec![0, 1, 2, 2, 3, 0],
        };
        assert_eq!(mesh_triangle_count(&mesh), 2);
    }
}
