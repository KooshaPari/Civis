//! Shared ground-height sampling for live attach clients (`live_scene`, `bevy_window`).

use std::collections::HashMap;

use civ_voxel::{ChunkId, MaterialId};

use crate::{decode_chunk_id, terrain::terrain_surface_y};

const CHUNK_EDGE: usize = 16;

/// Cached dense voxel payloads keyed by raw [`ChunkId`] bits.
#[derive(Debug, Default, Clone)]
pub struct ChunkVoxelCache {
    chunks: HashMap<u64, Vec<MaterialId>>,
}

impl ChunkVoxelCache {
    /// Creates an empty cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Stores or replaces the voxel payload for a chunk.
    pub fn insert(&mut self, chunk_id: ChunkId, voxels: Vec<MaterialId>) {
        if voxels.len() == CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE {
            self.chunks.insert(chunk_id.0, voxels);
        }
    }

    /// Borrow the underlying map (for iteration in samplers).
    #[must_use]
    pub fn chunks(&self) -> &HashMap<u64, Vec<MaterialId>> {
        &self.chunks
    }
}

fn voxel_index(ix: usize, iy: usize, iz: usize) -> usize {
    ix + iy * CHUNK_EDGE + iz * CHUNK_EDGE * CHUNK_EDGE
}

fn is_solid_voxel(material: MaterialId) -> bool {
    material.0 != 0
}

/// Top Y of the highest solid voxel in the world column at `(x, z)` from cached live chunks.
#[must_use]
pub fn live_voxel_surface_y(cache: &ChunkVoxelCache, x: f32, z: f32) -> Option<f32> {
    let edge_i = CHUNK_EDGE as i32;
    let edge_f = CHUNK_EDGE as f32;
    let cx = (x / edge_f).floor() as i32;
    let cz = (z / edge_f).floor() as i32;
    let ix = (x.floor() as i32).rem_euclid(edge_i) as usize;
    let iz = (z.floor() as i32).rem_euclid(edge_i) as usize;

    let mut best: Option<f32> = None;
    for (&chunk_raw, voxels) in cache.chunks() {
        if voxels.len() != CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE {
            continue;
        }
        let (chunk_cx, chunk_cy, chunk_cz) = decode_chunk_id(ChunkId(chunk_raw));
        if chunk_cx != cx || chunk_cz != cz {
            continue;
        }
        for iy in (0..CHUNK_EDGE).rev() {
            if is_solid_voxel(voxels[voxel_index(ix, iy, iz)]) {
                let surface_y = (chunk_cy * edge_i + iy as i32 + 1) as f32;
                best = Some(best.map_or(surface_y, |current| current.max(surface_y)));
                break;
            }
        }
    }
    best
}

/// Ground height at `(x, z)` from streamed voxels when available, else procedural terrain.
#[must_use]
pub fn live_ground_y(cache: &ChunkVoxelCache, x: f32, z: f32, offset: f32) -> f32 {
    live_voxel_surface_y(cache, x, z).unwrap_or_else(|| terrain_surface_y(x, z)) + offset
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode_chunk_id;

    #[test]
    fn voxel_column_surface_finds_top_solid() {
        let mut voxels = vec![MaterialId(0); CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE];
        voxels[voxel_index(4, 3, 5)] = MaterialId(1);
        let id = encode_chunk_id(0, 0, 0);
        let mut cache = ChunkVoxelCache::new();
        cache.insert(id, voxels);
        let y = live_voxel_surface_y(&cache, 4.5, 5.5).expect("surface");
        assert!((y - 4.0).abs() < f32::EPSILON);
    }

    #[test]
    fn live_ground_y_falls_back_to_terrain_without_chunks() {
        let cache = ChunkVoxelCache::new();
        let offset = 0.8;
        let y = live_ground_y(&cache, 64.0, 128.0, offset);
        let expected = terrain_surface_y(64.0, 128.0) + offset;
        assert!((y - expected).abs() < 0.01);
    }
}
