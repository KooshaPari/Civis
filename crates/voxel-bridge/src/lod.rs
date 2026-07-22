//! Public adapter for engine-agnostic voxel LOD planning.

pub use civ_voxel::lod::*;

use crate::ChunkId;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkDirty;

pub fn drain_dirty_chunks() -> Vec<ChunkId> {
    dirty_queue().lock().expect("dirty queue poisoned").drain(..).collect()
}

pub fn mark_lod_dirty(id: ChunkId) { mark(id); }
pub fn mark_storage_dirty(id: ChunkId) { mark(id); }

fn dirty_queue() -> &'static Mutex<Vec<ChunkId>> {
    static QUEUE: OnceLock<Mutex<Vec<ChunkId>>> = OnceLock::new();
    QUEUE.get_or_init(|| Mutex::new(Vec::new()))
}

fn mark(id: ChunkId) {
    let mut queue = dirty_queue().lock().expect("dirty queue poisoned");
    if !queue.contains(&id) { queue.push(id); }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dirty_marks_are_deduplicated_and_drained() {
        let id = ChunkId(41);
        mark_lod_dirty(id);
        mark_storage_dirty(id);
        assert_eq!(drain_dirty_chunks(), vec![id]);
        assert!(drain_dirty_chunks().is_empty());
    }
}
