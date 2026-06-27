//! FR-EMG-005: Diplomacy emergence oracle.
//!
//! Validates that at least one inter-faction stance change has occurred —
//! a `DiplomacyEvent` of any kind (TradeAgreement, Conflict, or Peace). A
//! non-empty event log confirms that the diplomacy phase ran, evaluated
//! faction-pair signals, and resolved at least one outcome (not merely neutral
//! inertia).
//!
//! Measurement: total `DiplomacyEvent`s accumulated across all ticks.
//! Threshold: ≥ 1 event after tick > 0.

use crate::{FeatureOracle, OracleVerdict};
use civ_engine::Simulation;

pub struct DiplomacyOracle;

impl FeatureOracle for DiplomacyOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-005"
    }

    fn check(&self, sim: &Simulation) -> OracleVerdict {
        let tick = sim.state.tick;
        let events = sim.diplomacy_events();
        let event_count = events.len();

        // Classify events so the detail string shows what actually happened.
        let trade_count = events
            .iter()
            .filter(|e| matches!(e.kind, civ_engine::DiplomacyKind::TradeAgreement))
            .count();
        let conflict_count = events
            .iter()
            .filter(|e| matches!(e.kind, civ_engine::DiplomacyKind::Conflict))
            .count();
        let peace_count = events
            .iter()
            .filter(|e| matches!(e.kind, civ_engine::DiplomacyKind::Peace))
            .count();

        let measured = event_count as f64;
        let threshold = if tick == 0 { 0.0 } else { 1.0 };
        let passed = tick == 0 || event_count >= 1;

        OracleVerdict {
            fr_id: self.fr_id().to_string(),
            passed,
            measured,
            threshold,
            detail: format!(
                "Diplomacy emergence: total_events={event_count} \
                 (trade={trade_count} conflict={conflict_count} peace={peace_count}) \
                 at tick={tick}"
            ),
        }
    }
}
