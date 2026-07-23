//! Public adapter for the validated voxel streaming-window contracts.
//!
//! The policy implementation is owned by `civ-voxel`; this crate keeps the
//! client-facing module path stable without maintaining a second stub model.

pub use civ_voxel::window::{
    ring_distance, ChunkState, EvictionKey, PolicyError, SimCohort, WindowPolicy,
};
pub use civ_voxel::window::ring_iter::RingIter;

pub mod io {
    pub use civ_voxel::window::io::*;
}

pub mod plan {
    pub use civ_voxel::window::plan::*;
}

pub mod ring_iter {
    pub use civ_voxel::window::ring_iter::*;
}
