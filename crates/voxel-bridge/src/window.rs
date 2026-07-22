//! TODO(FR): Streaming window and eviction policy module stub.

/// Chunk state in the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChunkState {
    /// TODO
    Resident,
}

/// Eviction key for priority-based eviction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EvictionKey;

/// Policy error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PolicyError;

/// Simulation cohort metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimCohort;

/// Window policy for streaming.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowPolicy;

/// Ring distance calculation.
pub fn ring_distance(_a: i32, _b: i32, _c: i32) -> u32 {
    _a.unsigned_abs()
        .max(_b.unsigned_abs())
        .max(_c.unsigned_abs())
}

/// IO submodule.
pub mod io {
    /// IO contract version.
    pub const IO_CONTRACT_VERSION: u32 = 1;

    /// Materialized snapshot.
    #[derive(Debug, Clone, PartialEq)]
    pub struct MaterializedSnapshot;

    /// IO contract.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct IoContract;
}

/// Plan submodule.
pub mod plan {
    /// Default prefetch ticks.
    pub const DEFAULT_PREFETCH_TICKS: u32 = 1;

    /// P99 sample cap.
    pub const P99_SAMPLE_CAP: usize = 100;

    /// Velocity chunks per tick.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct VelocityChunksPerTick;

    /// Scale report.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ScaleReport;

    /// Chunk offset iterator.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ChunkOffsetIter;

    /// Prefetch set for chunks.
    pub fn prefetch_set() -> Vec<(i32, i32, i32)> {
        vec![(0, 0, 0)]
    }
}

/// Ring iterator submodule.
pub mod ring_iter {
    /// Ring iterator over chunks.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct RingIter;
}
