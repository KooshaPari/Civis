//! Public adapter for the validated voxel streaming-window contracts.

pub use civ_voxel::window::ring_iter::RingIter;
pub use civ_voxel::window::{
    ring_distance, ChunkState, EvictionKey, PolicyError, SimCohort, WindowPolicy,
};

pub mod io {
    pub use civ_voxel::window::io::*;
}

pub mod plan {
    pub use civ_voxel::window::plan::*;
}

pub mod ring_iter {
    pub use civ_voxel::window::ring_iter::*;
}
