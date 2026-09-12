//! Climate and planetary subsystem for the simulation engine.
//!
//! This module contains types and helpers related to climate computation,
//! weather grids, and coastal water column management.

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
    /// Last y the water marker was written at.
    pub last_water_y: i64,
}

// ---- Simulation climate/planet methods (extracted from engine.rs) ----

use crate::engine::Simulation;
use civ_planet::{compute_climate, compute_weather, MoonConfig};
use civ_voxel::FIXED_SCALE;

impl Simulation {
    /// Planet phase - recompute climate and weather grid from the current tick,
    /// then apply the resulting tide offset to any registered coastal water
    /// columns (FR-CIV-PLANET-020, FR-CIV-PLANET-030).
    pub(crate) fn phase_planet(&mut self) {
        self.climate = compute_climate(self.state.tick, &self.planet, &self.moon);
        self.weather_grid = compute_weather(
            &self.climate,
            self.state.tick,
            self.weather_grid.len().max(1) as u32,
        );
        self.apply_tide_offset();
    }

    /// Register (or update) a coastal water column at horizontal `(x, z)` with
    /// sea-level baseline `base_y`. The column's water-marker voxel will be
    /// shifted vertically each tick by the climate `tide_offset` (FR-CIV-PLANET-020).
    ///
    /// Coordinates are fixed-point world units (see [`FIXED_SCALE`]). Calling
    /// this for an already-registered column resets its baseline, clears its
    /// prior marker when it is still water, and writes the new baseline marker.
    pub fn register_coastal_water_column(&mut self, x: i64, z: i64, base_y: i64) {
        let new_pos = WorldCoord { x, y: base_y, z };
        if let Some(previous) = self.coastal_columns.get(&(x, z)).copied() {
            if previous.last_water_y != base_y {
                let old_pos = WorldCoord {
                    x,
                    y: previous.last_water_y,
                    z,
                };
                if self.voxel.read(old_pos) == WATER_MARKER_MATERIAL {
                    self.push_voxel_write(old_pos, MaterialId(0));
                }
                self.push_voxel_write(new_pos, WATER_MARKER_MATERIAL);
            } else if self.voxel.read(new_pos) != WATER_MARKER_MATERIAL {
                self.push_voxel_write(new_pos, WATER_MARKER_MATERIAL);
            }
        } else {
            self.push_voxel_write(new_pos, WATER_MARKER_MATERIAL);
        }
        let column = CoastalColumn {
            base_y,
            last_water_y: base_y,
        };
        self.coastal_columns.insert((x, z), column);
    }

    /// Borrow the registered coastal water columns (for tests + tooling).
    #[must_use]
    pub fn coastal_column_count(&self) -> usize {
        self.coastal_columns.len()
    }

    /// Read the current water-level y for the column at `(x, z)`, if registered.
    #[must_use]
    pub fn coastal_water_level(&self, x: i64, z: i64) -> Option<i64> {
        self.coastal_columns.get(&(x, z)).map(|c| c.last_water_y)
    }

    /// Shift every registered coastal water-level voxel by the current
    /// `climate.tide_offset` (FR-CIV-PLANET-020). The offset is scaled into
    /// fixed-point world units, rounded deterministically, and applied through
    /// [`Simulation::push_voxel_write`] so replay and dirty events propagate normally
    /// (FR-CIV-VOXEL-002).
    ///
    /// For each column we clear the previously occupied water voxel (write
    /// `MaterialId(0)`) and write [`WATER_MARKER_MATERIAL`] at the new height.
    /// If the new height matches the old one we skip the redundant pair of
    /// writes to avoid emitting spurious dirty events.
    pub(crate) fn apply_tide_offset(&mut self) {
        if self.coastal_columns.is_empty() {
            return;
        }

        // Fixed-point conversion: `tide_offset` is a float amplitude in the
        // same world-unit space as the voxel grid; multiply by FIXED_SCALE and
        // round to the nearest integer for determinism. f32::round() is
        // deterministic per the IEEE-754 round-half-away-from-zero rule used
        // across our target platforms.
        let scale = FIXED_SCALE as f32;
        let offset_units = (self.climate.tide_offset * scale).round() as i64;

        // Collect updates first so we can mutate `self.voxel` and
        // `self.coastal_columns` without aliasing.
        let updates: Vec<((i64, i64), i64, i64)> = self
            .coastal_columns
            .iter()
            .map(|(&(x, z), column)| {
                let new_y = column.base_y.saturating_add(offset_units);
                ((x, z), column.last_water_y, new_y)
            })
            .collect();

        for ((x, z), prev_y, new_y) in updates {
            if prev_y == new_y {
                continue;
            }
            self.push_voxel_write(WorldCoord { x, y: prev_y, z }, MaterialId(0));
            self.push_voxel_write(WorldCoord { x, y: new_y, z }, WATER_MARKER_MATERIAL);
            if let Some(column) = self.coastal_columns.get_mut(&(x, z)) {
                column.last_water_y = new_y;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::save_bundle::CivSaveBundle;
    use tempfile::tempdir;

    #[test]
    fn tide_movement_survives_save_and_load() {
        let mut sim = Simulation::with_seed(72);
        sim.moon = MoonConfig {
            orbit_period_ticks: 4,
            tidal_amplitude: 1.0,
        };
        let (x, z, base_y) = (39, -19, 600);
        sim.register_coastal_water_column(x, z, base_y);
        sim.state.tick = 1;
        sim.phase_planet();
        let moved = WorldCoord {
            x,
            y: base_y + FIXED_SCALE,
            z,
        };
        assert_eq!(
            sim.voxel().read(WorldCoord { x, y: base_y, z }),
            MaterialId(0)
        );
        assert_eq!(sim.voxel().read(moved), WATER_MARKER_MATERIAL);
        let dir = tempdir().expect("tempdir");
        let save_path = dir.path().join("moved-coastal");
        CivSaveBundle::save_dir(&save_path, &sim).expect("save after tide movement");
        let mut loaded = CivSaveBundle::load_dir(&save_path).expect("load");
        assert_eq!(
            loaded.voxel().read(WorldCoord { x, y: base_y, z }),
            MaterialId(0)
        );
        assert_eq!(loaded.voxel().read(moved), WATER_MARKER_MATERIAL);
        assert_eq!(loaded.coastal_water_level(x, z), Some(moved.y));
        loaded.state.tick = 2;
        loaded.phase_planet();
        assert_eq!(loaded.voxel().read(moved), MaterialId(0));
        assert_eq!(
            loaded.voxel().read(WorldCoord { x, y: base_y, z }),
            WATER_MARKER_MATERIAL
        );
    }

    #[test]
    fn reregistering_a_moved_coastal_column_survives_save_and_the_next_tide() {
        let mut sim = Simulation::with_seed(73);
        sim.moon = MoonConfig {
            orbit_period_ticks: 4,
            tidal_amplitude: 1.0,
        };
        let (x, z, initial_base) = (40, -20, 700);
        let moved_base = initial_base + 3 * FIXED_SCALE;
        sim.register_coastal_water_column(x, z, initial_base);
        sim.state.tick = 1;
        sim.phase_planet();
        let old_marker = WorldCoord {
            x,
            y: initial_base + FIXED_SCALE,
            z,
        };
        sim.register_coastal_water_column(x, z, moved_base);
        let new_marker = WorldCoord {
            x,
            y: moved_base,
            z,
        };
        assert_eq!(sim.voxel().read(old_marker), MaterialId(0));
        assert_eq!(sim.voxel().read(new_marker), WATER_MARKER_MATERIAL);
        let dir = tempdir().expect("tempdir");
        let save_path = dir.path().join("reregistered-coastal");
        CivSaveBundle::save_dir(&save_path, &sim).expect("save");
        let mut loaded = CivSaveBundle::load_dir(&save_path).expect("load");
        assert_eq!(
            loaded.voxel().read(WorldCoord {
                x,
                y: initial_base,
                z
            }),
            MaterialId(0)
        );
        assert_eq!(loaded.voxel().read(old_marker), MaterialId(0));
        assert_eq!(loaded.voxel().read(new_marker), WATER_MARKER_MATERIAL);
        loaded.state.tick = 3;
        loaded.phase_planet();
        let next_marker = WorldCoord {
            x,
            y: moved_base - FIXED_SCALE,
            z,
        };
        assert_eq!(loaded.voxel().read(new_marker), MaterialId(0));
        assert_eq!(loaded.voxel().read(next_marker), WATER_MARKER_MATERIAL);
    }

    #[test]
    fn reregistering_preserves_an_intervening_authored_voxel() {
        let mut sim = Simulation::with_seed(74);
        sim.moon = MoonConfig {
            orbit_period_ticks: 4,
            tidal_amplitude: 1.0,
        };
        let (x, z, base_y) = (41, -21, 800);
        sim.register_coastal_water_column(x, z, base_y);
        sim.state.tick = 1;
        sim.phase_planet();
        let former_marker = WorldCoord {
            x,
            y: base_y + FIXED_SCALE,
            z,
        };
        sim.voxel_mut().write(former_marker, MaterialId(77));
        let new_base = base_y + 3 * FIXED_SCALE;
        sim.register_coastal_water_column(x, z, new_base);
        assert_eq!(sim.voxel().read(former_marker), MaterialId(77));
        assert_eq!(
            sim.voxel().read(WorldCoord { x, y: new_base, z }),
            WATER_MARKER_MATERIAL
        );
    }

    #[test]
    fn reregistering_an_unchanged_column_repairs_only_a_replaced_marker() {
        let mut sim = Simulation::with_seed(75);
        let pos = WorldCoord {
            x: 42,
            y: 900,
            z: -22,
        };
        sim.register_coastal_water_column(pos.x, pos.z, pos.y);
        let after_seed = sim.replay_log().events.len();
        sim.register_coastal_water_column(pos.x, pos.z, pos.y);
        assert_eq!(sim.replay_log().events.len(), after_seed);
        sim.voxel_mut().write(pos, MaterialId(77));
        let after_authored_write = sim.replay_log().events.len();
        sim.register_coastal_water_column(pos.x, pos.z, pos.y);
        assert_eq!(sim.voxel().read(pos), WATER_MARKER_MATERIAL);
        assert_eq!(sim.replay_log().events.len(), after_authored_write + 1);
    }
}
