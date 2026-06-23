//! Transport-safe mirror of the emergence sample payload.
//!
//! This type is intentionally small and self-contained so crates that
//! need to move emergence data across process boundaries can do so
//! without depending on the engine crate.

use serde::{Deserialize, Serialize};

/// Transport-safe snapshot of the emergence sample summary.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct EmergenceSampleSnapshot {
    /// Number of active agents in the sample window.
    pub agent_count: u32,
    /// Number of factions in the sample window.
    pub faction_count: u32,
    /// Resource entropy over the sampled population.
    pub resource_entropy: f32,
    /// Count of connected structures in the sample.
    pub structure_count: u32,
    /// Novelty-rate summary for the sample.
    pub novelty_rate: f32,
    /// Coupling strength summary for the sample.
    pub coupling_strength: f32,
    /// Engine tick the snapshot was captured at.
    pub tick: u64,
}

#[cfg(test)]
mod tests {
    use super::EmergenceSampleSnapshot;

    #[test]
    fn emergence_sample_snapshot_default_is_zeroed() {
        let snapshot = EmergenceSampleSnapshot::default();
        assert_eq!(snapshot.agent_count, 0);
        assert_eq!(snapshot.faction_count, 0);
        assert_eq!(snapshot.resource_entropy, 0.0);
        assert_eq!(snapshot.structure_count, 0);
        assert_eq!(snapshot.novelty_rate, 0.0);
        assert_eq!(snapshot.coupling_strength, 0.0);
        assert_eq!(snapshot.tick, 0);
    }
}
