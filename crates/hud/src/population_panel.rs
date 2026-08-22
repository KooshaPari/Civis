//! Population panel — aggregated population statistics for the HUD dashboard.
//!
//! Provides structured data types for population count, growth rate,
//! age demographics, and faction alignment breakdown. All fields are
//! serialisable to JSON for wire transport to any client (web / Bevy /
//! Godot / Unreal).
//!
//! Design contract:
//! 1. **Pure data, no engine.** No Bevy, no rendering, no systems.
//! 2. **Additive only.** This module does not modify any existing public
//!    surface.
//! 3. **Serialisation-safe.** Every field is `serde`-friendly.

use serde::{Deserialize, Serialize};

/// Age-band buckets used for demographic breakdowns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgeBand {
    /// Children (0-14 years).
    Child,
    /// Working-age adults (15-64 years).
    Adult,
    /// Elders (65+ years).
    Elder,
}

/// Population count within a single age band.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgeBandCount {
    /// The age band.
    pub band: AgeBand,
    /// Number of civilians in this band.
    pub count: u32,
}

/// Faction alignment breakdown — how many civilians belong to each faction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactionBreakdown {
    /// Faction identifier.
    pub faction_id: u32,
    /// Number of civilians aligned to this faction.
    pub count: u32,
}

/// Aggregated population statistics snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PopulationPanel {
    /// Total civilian count.
    pub total: u32,
    /// Per-tick growth rate (fractional, e.g. 0.02 = +2%).
    pub growth_rate: f32,
    /// Per-faction alignment breakdown.
    pub factions: Vec<FactionBreakdown>,
    /// Per-age-band demographic breakdown.
    pub demographics: Vec<AgeBandCount>,
}

impl PopulationPanel {
    /// Construct an empty panel with zero population.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            total: 0,
            growth_rate: 0.0,
            factions: Vec::new(),
            demographics: Vec::new(),
        }
    }

    /// Number of distinct factions represented.
    #[must_use]
    pub fn faction_count(&self) -> usize {
        self.factions.len()
    }

    /// Largest faction count, or `0` if no factions.
    #[must_use]
    pub fn dominant_faction_count(&self) -> u32 {
        self.factions.iter().map(|f| f.count).max().unwrap_or(0)
    }

    /// Fraction of population in a given age band (0.0-1.0).
    #[must_use]
    pub fn age_fraction(&self, band: AgeBand) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        let count = self
            .demographics
            .iter()
            .find(|d| d.band == band)
            .map(|d| d.count)
            .unwrap_or(0);
        count as f32 / self.total as f32
    }

    /// Population delta per tick derived from `growth_rate`.
    #[must_use]
    pub fn projected_delta(&self) -> f32 {
        self.total as f32 * self.growth_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_panel() -> PopulationPanel {
        PopulationPanel {
            total: 1000,
            growth_rate: 0.05,
            factions: vec![
                FactionBreakdown {
                    faction_id: 1,
                    count: 600,
                },
                FactionBreakdown {
                    faction_id: 2,
                    count: 400,
                },
            ],
            demographics: vec![
                AgeBandCount {
                    band: AgeBand::Child,
                    count: 200,
                },
                AgeBandCount {
                    band: AgeBand::Adult,
                    count: 650,
                },
                AgeBandCount {
                    band: AgeBand::Elder,
                    count: 150,
                },
            ],
        }
    }

    #[test]
    fn empty_panel_has_zero_total() {
        let p = PopulationPanel::empty();
        assert_eq!(p.total, 0);
        assert_eq!(p.growth_rate, 0.0);
        assert!(p.factions.is_empty());
        assert!(p.demographics.is_empty());
    }

    #[test]
    fn dominant_faction_count_returns_largest() {
        let p = sample_panel();
        assert_eq!(p.dominant_faction_count(), 600);
    }

    #[test]
    fn age_fraction_computes_correctly() {
        let p = sample_panel();
        assert!((p.age_fraction(AgeBand::Child) - 0.2).abs() < 1e-6);
        assert!((p.age_fraction(AgeBand::Adult) - 0.65).abs() < 1e-6);
        assert!((p.age_fraction(AgeBand::Elder) - 0.15).abs() < 1e-6);
    }

    #[test]
    fn age_fraction_returns_zero_for_empty_panel() {
        let p = PopulationPanel::empty();
        assert!((p.age_fraction(AgeBand::Adult) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn projected_delta_matches_growth_rate() {
        let p = sample_panel();
        assert!((p.projected_delta() - 50.0).abs() < 1e-6);
    }

    #[test]
    fn panel_round_trips_via_serde_json() {
        let p = sample_panel();
        let json = serde_json::to_string(&p).expect("serialize");
        let back: PopulationPanel = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(p.total, back.total);
        assert_eq!(p.factions.len(), back.factions.len());
        assert_eq!(p.demographics.len(), back.demographics.len());
    }
}
