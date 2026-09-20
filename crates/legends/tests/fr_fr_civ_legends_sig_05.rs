//! Tests for FR-CIV-LEGENDS-SIG-05 — Significance scoring
//!
//! Epic: FR-CIV-LEGENDS
//! Verifies the significance accumulator and scoring weights.

#[cfg(test)]
mod fr_fr_civ_legends_sig_05 {
    use civ_legends::significance::*;
    use civ_legends::model::{EventKind, Role};

    /// FR-CIV-LEGENDS-SIG-05: Significance accumulator starts empty.
    #[test]
    fn accumulator_starts_empty() {
        let acc = SignificanceAccumulator::new();
        assert!(acc.is_empty());
        assert_eq!(acc.len(), 0);
    }

    /// FR-CIV-LEGENDS-SIG-05: Recording an event increases accumulator count.
    #[test]
    fn recording_increases_count() {
        let mut acc = SignificanceAccumulator::new();
        let config = SignificanceConfig::default();
        let id = civ_legends::LegendEntityId(42);
        acc.record_event(
            id,
            civ_legends::Epoch(0),
            &EventKind::Death,
            &[Role::Leader],
            0.8,
            &config,
        );
        assert_eq!(acc.len(), 1);
        let sig = acc.get(id).expect("entity should exist");
        assert!(sig.score > 0.0);
        assert_eq!(sig.event_count, 1);
    }

    /// FR-CIV-LEGENDS-SIG-05: Leader role has higher weight than Witness.
    #[test]
    fn leader_weight_exceeds_witness() {
        assert!(Role::Leader.weight() > Role::Witness.weight());
    }

    /// FR-CIV-LEGENDS-SIG-05: Death events are weighted higher than Birth events.
    #[test]
    fn death_weighted_higher_than_birth() {
        use civ_legends::config::kind_weight;
        assert!(kind_weight(&EventKind::Death) >= kind_weight(&EventKind::Birth));
    }

    /// FR-CIV-LEGENDS-SIG-05: Significance is capped at max_significance.
    #[test]
    fn significance_capped_at_max() {
        let mut acc = SignificanceAccumulator::new();
        let mut config = SignificanceConfig::default();
        config.max_significance = 0.5;
        let id = civ_legends::LegendEntityId(0);
        for tick in 0..100u64 {
            acc.record_event(
                id,
                civ_legends::Epoch(tick),
                &EventKind::Death,
                &[Role::Leader],
                1.0,
                &config,
            );
        }
        let sig = acc.get(id).expect("entity should exist");
        assert!(
            sig.score <= config.max_significance,
            "score {} should be capped at {}",
            sig.score,
            config.max_significance
        );
    }
}
