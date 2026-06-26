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

/// Compute the distance-based LOD level for a chunk at a given camera position.
///
/// Given a camera/observer position and a chunk coordinate, computes the
/// Euclidean distance in world space and returns the appropriate LOD level
/// for mesh tessellation. Higher LOD levels indicate coarser geometry.
///
/// # Arguments
/// * `camera_pos` - Camera position in world space (floating-point).
/// * `chunk_center` - Chunk center position in world space (floating-point).
/// * `scale` - Voxel scale multiplier for consistent distance scaling.
/// * `policy` - LOD policy defining distance thresholds and max LOD level.
///
/// # Returns
/// The computed LOD level (0 = full detail, higher = coarser).
///
/// # Example
/// ```ignore
/// let camera = glam::Vec3::new(0.0, 0.0, 0.0);
/// let chunk_center = glam::Vec3::new(10.0, 0.0, 0.0);
/// let scale = VoxelScaleMultiplier(8.0);
/// let policy = LodPolicy::default();
/// let lod = compute_chunk_lod(camera, chunk_center, scale, policy);
/// assert_eq!(lod, LodLevel(0)); // Near chunk gets LOD 0
/// ```
#[must_use]
pub fn compute_chunk_lod(
    camera_pos: glam::Vec3,
    chunk_center: glam::Vec3,
    scale: VoxelScaleMultiplier,
    policy: LodPolicy,
) -> LodLevel {
    let distance = camera_pos.distance(chunk_center);
    select_mesh_detail_level(distance, scale, policy)
}

/// Represents a downsampled chunk snapshot for distant rendering.
///
/// For far chunks at higher LOD levels, a downsampled representation can
/// reduce triangle and vertex counts. This struct encodes which voxels
/// to sample from the original chunk (stride = 2^lod).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DownsampledChunk {
    /// LOD level indicating stride (2^lod_level skips between sampled voxels).
    pub lod_level: LodLevel,
    /// Sampling stride: 1 << lod_level (e.g., LOD 2 = stride 4).
    pub stride: usize,
    /// Expected reduced count of sampled voxels (~(16/stride)^3 for a 16³ chunk).
    pub expected_sample_count: usize,
}

impl DownsampledChunk {
    /// Create a downsampled chunk representation for the given LOD level.
    ///
    /// # Arguments
    /// * `lod_level` - LOD level (0 means full detail, no downsampling).
    /// * `original_edge_size` - Original chunk edge size in voxels (typically 16).
    ///
    /// # Returns
    /// A `DownsampledChunk` struct encoding stride and sample count.
    ///
    /// # Example
    /// ```ignore
    /// // Full detail: no downsampling
    /// let chunk = DownsampledChunk::new(LodLevel(0), 16);
    /// assert_eq!(chunk.stride, 1);
    /// assert_eq!(chunk.expected_sample_count, 4096); // 16³
    ///
    /// // LOD 1: every 2nd voxel → 8³ = 512 samples
    /// let chunk = DownsampledChunk::new(LodLevel(1), 16);
    /// assert_eq!(chunk.stride, 2);
    /// assert_eq!(chunk.expected_sample_count, 512);
    /// ```
    #[must_use]
    pub fn new(lod_level: LodLevel, original_edge_size: usize) -> Self {
        let stride = 1_usize.saturating_shl(lod_level.0 as u32);
        let sampled_edge = (original_edge_size + stride - 1) / stride;
        let expected_sample_count = sampled_edge.pow(3);
        Self {
            lod_level,
            stride,
            expected_sample_count,
        }
    }

    /// Determine if a voxel at the given local coordinate should be sampled.
    ///
    /// For a downsampled chunk, only voxels whose coordinates are divisible
    /// by the stride should be included in the sampled set.
    ///
    /// # Arguments
    /// * `local_x`, `local_y`, `local_z` - Local voxel coordinates within the chunk.
    ///
    /// # Returns
    /// `true` if the voxel at (local_x, local_y, local_z) should be sampled.
    #[must_use]
    pub fn should_sample(&self, local_x: usize, local_y: usize, local_z: usize) -> bool {
        self.stride == 1
            || (local_x % self.stride == 0
                && local_y % self.stride == 0
                && local_z % self.stride == 0)
    }

    /// Estimate triangle count for a mesh of this downsampled chunk.
    ///
    /// A rough heuristic: ~6 faces per cubic voxel, scaled by density.
    /// (This is a conservative estimate; actual triangle count may vary
    /// based on greedy meshing compression.)
    #[must_use]
    pub fn estimate_triangle_count(&self) -> usize {
        self.expected_sample_count * 6 / 2 // Rough estimate after greedy compression
    }
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

    #[test]
    fn compute_chunk_lod_near_chunk_returns_full_detail() {
        let camera_pos = glam::Vec3::new(0.0, 0.0, 0.0);
        let chunk_center = glam::Vec3::new(1.0, 0.0, 0.0);
        let scale = VoxelScaleMultiplier::default();
        let policy = LodPolicy::default();

        let lod = compute_chunk_lod(camera_pos, chunk_center, scale, policy);
        assert_eq!(lod, LodLevel(0), "Near chunk should have LOD 0 (full detail)");
    }

    #[test]
    fn compute_chunk_lod_far_chunk_returns_higher_lod() {
        let camera_pos = glam::Vec3::new(0.0, 0.0, 0.0);
        let chunk_center = glam::Vec3::new(1000.0, 0.0, 0.0);
        let scale = VoxelScaleMultiplier::default();
        let policy = LodPolicy::default();

        let lod = compute_chunk_lod(camera_pos, chunk_center, scale, policy);
        assert!(
            lod.0 > 0,
            "Far chunk should have LOD > 0 (coarser detail) but got LOD {}",
            lod.0
        );
    }

    #[test]
    fn downsampled_chunk_lod_0_has_full_stride() {
        let chunk = DownsampledChunk::new(LodLevel(0), 16);
        assert_eq!(chunk.stride, 1);
        assert_eq!(chunk.expected_sample_count, 16 * 16 * 16);
    }

    #[test]
    fn downsampled_chunk_lod_1_halves_samples() {
        let chunk = DownsampledChunk::new(LodLevel(1), 16);
        assert_eq!(chunk.stride, 2);
        assert_eq!(chunk.expected_sample_count, 8 * 8 * 8);
        assert_eq!(chunk.expected_sample_count, 512);
    }

    #[test]
    fn downsampled_chunk_lod_2_quarters_samples() {
        let chunk = DownsampledChunk::new(LodLevel(2), 16);
        assert_eq!(chunk.stride, 4);
        assert_eq!(chunk.expected_sample_count, 4 * 4 * 4);
        assert_eq!(chunk.expected_sample_count, 64);
    }

    #[test]
    fn downsampled_chunk_lod_3_eighths_samples() {
        let chunk = DownsampledChunk::new(LodLevel(3), 16);
        assert_eq!(chunk.stride, 8);
        assert_eq!(chunk.expected_sample_count, 2 * 2 * 2);
        assert_eq!(chunk.expected_sample_count, 8);
    }

    #[test]
    fn downsampled_chunk_should_sample_lod_0_samples_all() {
        let chunk = DownsampledChunk::new(LodLevel(0), 16);
        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    assert!(
                        chunk.should_sample(x, y, z),
                        "LOD 0 should sample voxel ({}, {}, {})",
                        x,
                        y,
                        z
                    );
                }
            }
        }
    }

    #[test]
    fn downsampled_chunk_should_sample_lod_1_alternates() {
        let chunk = DownsampledChunk::new(LodLevel(1), 16);
        assert!(chunk.should_sample(0, 0, 0));
        assert!(chunk.should_sample(2, 0, 0));
        assert!(chunk.should_sample(0, 4, 0));
        assert!(!chunk.should_sample(1, 0, 0));
        assert!(!chunk.should_sample(0, 1, 0));
        assert!(!chunk.should_sample(1, 1, 1));
    }

    #[test]
    fn downsampled_chunk_should_sample_lod_2_quaternary() {
        let chunk = DownsampledChunk::new(LodLevel(2), 16);
        assert!(chunk.should_sample(0, 0, 0));
        assert!(chunk.should_sample(4, 4, 4));
        assert!(chunk.should_sample(8, 8, 8));
        assert!(!chunk.should_sample(1, 0, 0));
        assert!(!chunk.should_sample(0, 2, 0));
        assert!(!chunk.should_sample(3, 3, 3));
    }

    #[test]
    fn downsampled_chunk_estimate_triangle_count_decreases_with_lod() {
        let chunk_lod0 = DownsampledChunk::new(LodLevel(0), 16);
        let chunk_lod1 = DownsampledChunk::new(LodLevel(1), 16);
        let chunk_lod2 = DownsampledChunk::new(LodLevel(2), 16);

        let tri_lod0 = chunk_lod0.estimate_triangle_count();
        let tri_lod1 = chunk_lod1.estimate_triangle_count();
        let tri_lod2 = chunk_lod2.estimate_triangle_count();

        assert!(tri_lod0 > tri_lod1, "LOD 0 should have more triangles than LOD 1");
        assert!(tri_lod1 > tri_lod2, "LOD 1 should have more triangles than LOD 2");
    }

    #[test]
    fn downsampled_chunk_is_deterministic() {
        let chunk1 = DownsampledChunk::new(LodLevel(1), 16);
        let chunk2 = DownsampledChunk::new(LodLevel(1), 16);

        assert_eq!(chunk1.stride, chunk2.stride);
        assert_eq!(chunk1.expected_sample_count, chunk2.expected_sample_count);

        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    assert_eq!(
                        chunk1.should_sample(x, y, z),
                        chunk2.should_sample(x, y, z)
                    );
                }
            }
        }
    }

    #[test]
    fn compute_chunk_lod_respects_scale_invariance() {
        let policy = LodPolicy::default();
        let camera_pos = glam::Vec3::ZERO;
        let chunk_center_a = glam::Vec3::new(64.0 * 8.0, 0.0, 0.0);
        let chunk_center_b = glam::Vec3::new(64.0 * 16.0, 0.0, 0.0);

        let lod_a = compute_chunk_lod(camera_pos, chunk_center_a, VoxelScaleMultiplier(8.0), policy);
        let lod_b =
            compute_chunk_lod(camera_pos, chunk_center_b, VoxelScaleMultiplier(16.0), policy);

        assert_eq!(
            lod_a, lod_b,
            "Scale-invariant distances should yield same LOD"
        );
    }
}
