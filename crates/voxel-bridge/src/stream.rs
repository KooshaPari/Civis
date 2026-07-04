//! TODO(FR): Chunk streaming module stub.

/// Chunk edge size constant.
pub const CHUNK_EDGE: usize = 16;

/// Chunk edge size as i32.
pub const CHUNK_EDGE_I32: i32 = 16;

/// Chunk store port.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkStorePort;

/// File system chunk store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FsChunkStore;

/// Stream configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamConfig;

/// Streaming statistics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamStats;

/// Streaming world instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamingWorld;

/// World generator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldGen;
