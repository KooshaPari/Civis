//! Public seam between voxel simulation and streaming/render clients.
//!
//! The individual feature modules are intentionally small contracts while
//! their implementations are being migrated from the historical prototype.

pub use civ_voxel::{ChunkId, MaterialId};

pub mod boundary;
pub mod fluid_ca;
pub mod hud;
pub mod lod;
pub mod material;
pub mod material_pbr;
pub mod reactions;
pub mod scale_budget;
pub mod stream;
pub mod window;
pub mod worldgen;
