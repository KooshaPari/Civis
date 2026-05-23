//! civ-bevy-ref library surface.
//!
//! Splits cleanly into two parts:
//!
//! - **Always compiled** — pure converters and helpers that turn the
//!   engine-neutral [`civ_voxel::MeshBuffer`] into engine-native vertex
//!   arrays. Currently this just re-exposes the kernel `MeshBuffer` and adds
//!   small utility shapes.
//! - **`bevy` feature** — the Bevy renderer (`pub mod bevy_render`). Pulls
//!   Bevy 0.14 behind an optional feature set. Off by default so the workspace
//!   build stays fast for CI / agent-driven smoke runs.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub use civ_voxel::{CubicMesher, MaterialId, MeshBuffer, MeshVertex, VoxelWorld, WorldCoord};

/// Engine-neutral camera placement helper. The actual renderer uses this to
/// position a chase / orbit camera around a voxel scene.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraTarget {
    /// Centre of the scene in world units.
    pub centre: [f32; 3],
    /// Distance from the centre to place the camera.
    pub distance: f32,
}

impl Default for CameraTarget {
    fn default() -> Self {
        Self {
            centre: [0.0, 0.0, 0.0],
            distance: 32.0,
        }
    }
}

#[cfg(feature = "bevy")]
pub mod bevy_render;

#[cfg(test)]
mod tests {
    use super::*;

    /// Always-on smoke: the public surface compiles + default camera target is
    /// sensible.
    #[test]
    fn default_camera_target_is_sensible() {
        let t = CameraTarget::default();
        assert!(t.distance > 0.0);
    }

    /// MeshBuffer re-export is callable.
    #[test]
    fn mesh_buffer_default_is_empty() {
        let m = MeshBuffer::default();
        assert!(m.vertices.is_empty());
        assert!(m.indices.is_empty());
    }
}
