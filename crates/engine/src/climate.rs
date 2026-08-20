//! Climate and planetary subsystem for the simulation engine.
//!
//! This module contains types and helpers related to climate computation,
//! weather grids, and coastal water column management. The actual `phase_planet`
//! method remains in `engine.rs` as it requires `&mut Simulation` access.

use civ_planet::{Climate, WeatherCell};
use civ_voxel::material::WATER;
use civ_voxel::{MaterialId, WorldCoord};
use serde::{Deserialize, Serialize};

/// Water marker material used for coastal tide voxel writes.
pub const WATER_MARKER_MATERIAL: MaterialId = WATER;

/// A coastal water column registered with the engine. Each column anchors a
/// single water-marker voxel that shifts vertically with the climate tide
/// offset every tick (FR-CIV-PLANET-020). Iteration order is deterministic
/// because columns live in a [`BTreeMap`](std::collections::BTreeMap).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoastalColumn {
    /// Sea-level y in fixed-point world units.
    pub base_y: i64,
    /// Last y the water marker was written at (so we can clear it before
    /// writing the new level — preserves FR-CIV-VOXEL-002 dirty-event
    /// invariants by going through `VoxelWorld::write`).
    pub last_water_y: i64,
}
