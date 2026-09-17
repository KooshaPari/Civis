//! Tests for FR-CIV-EMERGENCE-011 — sample snapshot.
//!
//! Epic: FR-CIV-EMERGENCE
//! Status: IMPLEMENTED

#[cfg(test)]
mod fr_fr_civ_emergence_011 {
    use civ_emergence_metrics::sample_snapshot::EmergenceSample;

    #[test]
    fn verify_fr_civ_emergence_011_basic() {
        let sample = EmergenceSample::default();
        assert_eq!(sample.tick, 0, "default sample tick should be 0");
    }

    #[test]
    fn sample_serializes_roundtrip() {
        use serde_json;
        let sample = EmergenceSample::default();
        let json = serde_json::to_string(&sample).expect("serialize");
        let back: EmergenceSample = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(sample.tick, back.tick);
    }
}
