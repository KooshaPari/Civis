//! Tests for FR-CIV-PERF-017
//!
//! Epic: FR-CIV-PERF
//!
//! This test file verifies FR FR-CIV-PERF-017: Fixed-point serialization roundtrip.

#[cfg(test)]
mod fr_fr_civ_perf_017 {
    /// Verify FR-CIV-PERF-017: Fixed-point values survive JSON roundtrip.
    #[test]
    fn verify_fr_civ_perf_017_basic() {
        use civ_engine::Fixed;
        let mut ws = civ_engine::WorldState::default();
        ws.energy_budget_joules = Fixed::from_num(5_000_000i64);
        let json = serde_json::to_string(&ws).expect("serialize");
        let ws2: civ_engine::WorldState = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(ws.energy_budget_joules, ws2.energy_budget_joules);
    }

    /// Verify Fixed::from_num and to_num roundtrip.
    #[test]
    fn fixed_from_num_roundtrip() {
        use civ_engine::Fixed;
        let v: i64 = 42_000;
        let f = Fixed::from_num(v);
        let back: i64 = f.to_num();
        assert_eq!(back, v);
    }
}
