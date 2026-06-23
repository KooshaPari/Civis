//! CIV-0101 two-zoom level-of-detail policy (stub).
//!
//! Zoom transitions are view-only projections; simulation truth state is
//! unchanged. Entity tick cadence follows deterministic modulo scheduling so
//! gestalt (Cold) tiers stay aligned with Hot tiers at shared sync ticks.

pub use civ_agents::LodTier;

/// Strategic (region) vs operational (district / hex) zoom levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZoomLevel {
    /// Region aggregates — macro governance view.
    Strategic,
    /// District / hex detail — tactical view.
    Operational,
}

/// Per-tier tick cadence for simulation fidelity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LodPolicy {
    /// Warm tier ticks every N simulation ticks.
    pub warm_cadence: u64,
    /// Cold (gestalt) tier ticks every N simulation ticks.
    pub cold_cadence: u64,
}

impl Default for LodPolicy {
    fn default() -> Self {
        Self {
            warm_cadence: 4,
            cold_cadence: 16,
        }
    }
}

impl LodPolicy {
    /// Modulo divisor for the given fidelity tier.
    pub fn cadence_for(self, tier: LodTier) -> u64 {
        match tier {
            LodTier::Hot => 1,
            LodTier::Warm => self.warm_cadence,
            LodTier::Cold => self.cold_cadence,
        }
    }
}

/// Return whether an entity at `tier` should simulate on `tick`.
///
/// Deterministic: Hot every tick; Warm/Cold on `tick % cadence == 0`.
pub fn should_tick_entity(tick: u64, tier: LodTier) -> bool {
    should_tick_entity_with_policy(tick, tier, LodPolicy::default())
}

/// Same as [`should_tick_entity`] with an explicit cadence policy.
pub fn should_tick_entity_with_policy(tick: u64, tier: LodTier, policy: LodPolicy) -> bool {
    match tier {
        LodTier::Hot => true,
        LodTier::Warm => tick % policy.warm_cadence == 0,
        LodTier::Cold => tick % policy.cold_cadence == 0,
    }
}

/// Roll district populations into a region summary (FR-LOD-002 stub).
pub fn aggregate_strategic(district_populations: &[u32]) -> u32 {
    district_populations.iter().sum()
}

/// Hex-cell snapshot exposed at operational zoom (FR-LOD-004 stub).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HexCellSnapshot {
    /// Population at this hex.
    pub population: u32,
    /// Resource stock at this hex.
    pub resources: u32,
}

/// Build an operational hex view from micro state (FR-LOD-004 stub).
pub fn operational_hex_snapshot(population: u32, resources: u32) -> HexCellSnapshot {
    HexCellSnapshot {
        population,
        resources,
    }
}

/// Project zoom level without mutating simulation tick state (FR-LOD-003 stub).
pub fn project_zoom(state_tick: u64, zoom: ZoomLevel) -> (u64, ZoomLevel) {
    (state_tick, zoom)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-LOD-001 — strategic and operational zoom levels are defined.
    #[test]
    fn two_levels_defined() {
        let levels = [ZoomLevel::Strategic, ZoomLevel::Operational];
        assert_eq!(levels.len(), 2);
        assert_ne!(levels[0], levels[1]);
    }

    /// FR-LOD-002 — strategic view aggregates district data.
    #[test]
    fn strategic_aggregation() {
        assert_eq!(aggregate_strategic(&[100, 200, 50]), 350);
        assert_eq!(aggregate_strategic(&[]), 0);
    }

    /// FR-LOD-003 — zoom transitions do not alter simulation tick state.
    #[test]
    fn transition_no_state_mutation() {
        let tick = 42_u64;
        let (strategic_tick, _) = project_zoom(tick, ZoomLevel::Strategic);
        let (operational_tick, _) = project_zoom(tick, ZoomLevel::Operational);
        assert_eq!(strategic_tick, tick);
        assert_eq!(operational_tick, tick);
    }

    /// FR-LOD-004 — operational view exposes hex-cell resource and population data.
    #[test]
    fn operational_hex_data_visible() {
        let cell = operational_hex_snapshot(12, 500);
        assert_eq!(cell.population, 12);
        assert_eq!(cell.resources, 500);
    }

    /// Gestalt cadence: Cold sync ticks are a subset of Hot ticks (no divergence).
    #[test]
    fn gestalt_no_divergence() {
        for tick in 0..64 {
            if should_tick_entity(tick, LodTier::Cold) {
                assert!(should_tick_entity(tick, LodTier::Hot));
            }
        }
    }

    #[test]
    fn should_tick_entity_respects_modulo_cadence() {
        assert!(should_tick_entity(1, LodTier::Hot));
        assert!(should_tick_entity(4, LodTier::Warm));
        assert!(!should_tick_entity(5, LodTier::Warm));
        assert!(should_tick_entity(16, LodTier::Cold));
        assert!(!should_tick_entity(17, LodTier::Cold));
    }

    #[test]
    fn lod_policy_default_matches_agents_cadence() {
        let policy = LodPolicy::default();
        assert_eq!(policy.cadence_for(LodTier::Hot), 1);
        assert_eq!(policy.cadence_for(LodTier::Warm), 4);
        assert_eq!(policy.cadence_for(LodTier::Cold), 16);
    }
}
