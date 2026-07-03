//! TODO(FR): Boundary module stub for adaptive voxel clipping.

/// Boundary configuration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundaryConfig;

/// Boundary face identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoundaryFace {
    /// TODO
    Front,
}

/// Boundary mode selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryMode {
    /// TODO
    Clamp,
}

/// 3D bounding box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds3 {
    /// Minimum corner.
    pub min: [f32; 3],
    /// Maximum corner.
    pub max: [f32; 3],
}
