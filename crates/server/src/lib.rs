//! civ-server library — exposes the 3D-extension protocol bridge that consumers
//! (renderers, replay tools) use to convert `Simulation::last_tick_voxel_events`
//! into `civ-protocol-3d` frames.
//!
//! The eventual WebSocket bridge lives here too; for now this crate ships the
//! frame builders and a binary that prints determinism metrics.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod voxel_frame_builder;

pub use voxel_frame_builder::{build_voxel_delta_frame, VoxelFrameBuilderError};
