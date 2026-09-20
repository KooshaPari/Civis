//! Tests for FR-CIV-LEGENDS-PERSIST-11 — Persistence
//!
//! Epic: FR-CIV-LEGENDS
//! Verifies the significance accumulator can be serialized.

#[cfg(test)]
mod fr_fr_civ_legends_persist_11 {
    use civ_legends::significance::*;

    /// FR-CIV-LEGENDS-PERSIST-11: SignificanceAccumulator roundtrips through JSON.
    #[test]
    fn significance_accumulator_roundtrips() {
        let mut acc = SignificanceAccumulator::new();
        let config = SignificanceConfig::default();
        let id = civ_legends::LegendEntityId(42);
        acc.record_event(
            id,
            civ_legends::Epoch(0),
            &civ_legends::EventKind::Battle,
            &[civ_legends::Role::Leader],
            0.8,
            &config,
        );
        let json = serde_json::to_string(&acc).expect("serialize");
        let restored: SignificanceAccumulator = serde_json::from_str(&json).expect("deserialize");
        let sig = restored.get(id).expect("entity should exist");
        assert!(sig.score > 0.0);
        assert_eq!(sig.event_count, 1);
    }

    /// FR-CIV-LEGENDS-PERSIST-11: Empty accumulator roundtrips.
    #[test]
    fn empty_accumulator_roundtrips() {
        let acc = SignificanceAccumulator::new();
        let json = serde_json::to_string(&acc).expect("serialize");
        let restored: SignificanceAccumulator = serde_json::from_str(&json).expect("deserialize");
        assert!(restored.is_empty());
    }
}
