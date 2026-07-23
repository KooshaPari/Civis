//! Public adapter for deterministic voxel streaming.
//!
//! Storage, seeded regeneration, LRU eviction, and LOD selection are owned by
//! `civ-voxel`. Re-exporting them here gives clients one bridge dependency and
//! prevents the adapter from drifting into a second implementation.

pub use civ_voxel::stream::{
    ChunkStorePort, FsChunkStore, StreamConfig, StreamStats, StreamingWorld, WorldGen, CHUNK_EDGE,
    CHUNK_EDGE_I32,
};
