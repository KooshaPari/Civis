//! Tests for FR-CIV-PERF-018
//!
//! Epic: FR-CIV-PERF
//!
//! This test file verifies FR FR-CIV-PERF-018: WorldState BTreeMap determinism.

#[cfg(test)]
mod fr_fr_civ_perf_018 {
    /// Verify FR-CIV-PERF-018: BTreeMap iteration order is deterministic.
    #[test]
    fn verify_fr_civ_perf_018_basic() {
        let mut ws = civ_engine::WorldState::default();
        ws.factions.insert(100, "Alpha".to_string());
        ws.factions.insert(200, "Beta".to_string());
        ws.factions.insert(300, "Gamma".to_string());
        let keys: Vec<_> = ws.factions.keys().copied().collect();
        // BTreeMap iterates in sorted key order
        // Verify the three new keys are in sorted order within the full map
        let idx_100 = keys.iter().position(|&k| k == 100).unwrap();
        let idx_200 = keys.iter().position(|&k| k == 200).unwrap();
        let idx_300 = keys.iter().position(|&k| k == 300).unwrap();
        assert!(
            idx_100 < idx_200 && idx_200 < idx_300,
            "BTreeMap must iterate in sorted key order"
        );
        // Full keys must be sorted
        for w in keys.windows(2) {
            assert!(w[0] < w[1], "keys must be ascending");
        }
    }

    /// Verify serialization preserves BTreeMap order.
    #[test]
    fn serialization_preserves_order() {
        let mut ws = civ_engine::WorldState::default();
        ws.factions.insert(100, "E".to_string());
        ws.factions.insert(200, "A".to_string());
        ws.factions.insert(300, "C".to_string());
        let json = serde_json::to_string(&ws).expect("serialize");
        let ws2: civ_engine::WorldState = serde_json::from_str(&json).expect("deserialize");
        let keys: Vec<_> = ws2.factions.keys().copied().collect();
        // All keys must be sorted
        for w in keys.windows(2) {
            assert!(w[0] < w[1], "BTreeMap serialization must preserve sorted order");
        }
    }
}
