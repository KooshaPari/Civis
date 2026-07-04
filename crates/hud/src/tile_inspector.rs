//! HOLO tile inspector (read-only projection).
//!
//! Per `docs/design/SAVELOAD_HUD_PLAN.md` §4.3 — opens on tool-arm and on
//! `Q` press; closes when the save panel opens (density rule). The inspector
//! is the **second** HUD holo surface; the first is the in-world brush ring.
//! Per `ui-design-language.md:469-471`, never more than two holo surfaces.

use serde::{Deserialize, Serialize};

/// Sentinel for "no cell selected" (the inspector is closed or no probe
/// landed on the watch event bus).
pub const CELL_NONE: Option<(i32, i32, i32)> = None;

/// Read-only inspector projection. All fields are **measured**, never
/// invented — charter: emergence, not scripted strings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileInspector {
    /// `(x, y, z)` of the probed cell, or `None` when closed.
    pub cell: Option<(i32, i32, i32)>,
    /// Voxel height at the cell (terrain heightfield).
    pub height: Option<u32>,
    /// Material id at the cell (rock / soil / sand / …).
    pub material: Option<String>,
    /// Biome id at the cell (temperate / tundra / desert / …).
    pub biome: Option<String>,
    /// Slope at the cell (0.0 = flat, 1.0 = vertical).
    pub slope: Option<f32>,
    /// Number of agents occupying the cell.
    pub agent_count: Option<u32>,
    /// Faction id of the dominant agent (if any).
    pub faction: Option<String>,
    /// Aggregated mood value (0.0–1.0).
    pub mood: Option<f32>,
}

impl Default for TileInspector {
    fn default() -> Self {
        Self::closed()
    }
}

impl TileInspector {
    /// Closed / no cell probed state.
    #[must_use]
    pub const fn closed() -> Self {
        Self {
            cell: None,
            height: None,
            material: None,
            biome: None,
            slope: None,
            agent_count: None,
            faction: None,
            mood: None,
        }
    }

    /// True when the inspector is currently open (has a probed cell).
    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.cell.is_some()
    }

    /// Open the inspector on a probed cell (clears any previous data).
    pub fn open_on(&mut self, cell: (i32, i32, i32)) {
        self.cell = Some(cell);
        // Clear measured fields; host client populates them via a follow-up
        // sim.snapshot probe.
        self.height = None;
        self.material = None;
        self.biome = None;
        self.slope = None;
        self.agent_count = None;
        self.faction = None;
        self.mood = None;
    }

    /// Apply measured substrate values (called by host after a snapshot probe).
    pub fn fill_from_snapshot(
        &mut self,
        height: u32,
        material: impl Into<String>,
        biome: impl Into<String>,
        slope: f32,
    ) {
        self.height = Some(height);
        self.material = Some(material.into());
        self.biome = Some(biome.into());
        self.slope = Some(slope);
    }

    /// Apply agent-derived values (called when an agent presence snapshot
    /// is available).
    pub fn fill_agents(
        &mut self,
        agent_count: u32,
        faction: Option<String>,
        mood: Option<f32>,
    ) {
        self.agent_count = Some(agent_count);
        self.faction = faction;
        self.mood = mood;
    }

    /// Close the inspector (called when the save panel opens).
    pub fn close(&mut self) {
        *self = Self::closed();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_closed() {
        let i = TileInspector::default();
        assert!(!i.is_open());
        assert_eq!(i.cell, None);
    }

    #[test]
    fn open_then_close() {
        let mut i = TileInspector::default();
        i.open_on((3, 7, 1));
        assert!(i.is_open());
        assert_eq!(i.cell, Some((3, 7, 1)));
        i.close();
        assert!(!i.is_open());
    }

    #[test]
    fn fill_from_snapshot_records_measured_values() {
        let mut i = TileInspector::default();
        i.open_on((12, 7, 3));
        i.fill_from_snapshot(14, "Rock", "Temperate", 0.7);
        assert_eq!(i.height, Some(14));
        assert_eq!(i.material.as_deref(), Some("Rock"));
        assert_eq!(i.biome.as_deref(), Some("Temperate"));
        assert_eq!(i.slope, Some(0.7));
        // agents stay unknown until a separate probe.
        assert_eq!(i.agent_count, None);
    }
}