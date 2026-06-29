//! TODO(FR): LOD management module stub.

use crate::ChunkId;

/// Chunk dirty event type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkDirty;

/// Drain dirty chunks from the world.
pub fn drain_dirty_chunks() -> Vec<ChunkId> {
    Vec::new()
}

/// Mark an LOD chunk as dirty.
pub fn mark_lod_dirty(_id: ChunkId) {
    // TODO(FR): implement
}

/// Mark storage as dirty.
pub fn mark_storage_dirty(_id: ChunkId) {
    // TODO(FR): implement
}
