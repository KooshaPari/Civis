//! Tests for FR-CIV-PERF-WEB-001
//!
//! Epic: FR-CIV-PERF-WEB
//!
//! This test file verifies FR FR-CIV-PERF-WEB-001: Web spectator view payload.

#[cfg(test)]
mod fr_fr_civ_perf_web_001 {
    /// Verify FR-CIV-PERF-WEB-001: SpectatorView serializes to JSON.
    #[test]
    fn verify_fr_civ_perf_web_001_basic() {
        let sim = civ_engine::Simulation::with_seed(42u64);
        let view = sim.spectator_view();
        let json = serde_json::to_string(&view).expect("serialize spectator view");
        assert!(
            json.len() > 100,
            "spectator view JSON should be substantial, got {} bytes",
            json.len()
        );
    }

    /// Verify spectator view JSON contains expected fields.
    #[test]
    fn spectator_view_json_structure() {
        let sim = civ_engine::Simulation::with_seed(42u64);
        let view = sim.spectator_view();
        let json = serde_json::to_string(&view).expect("serialize");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("parse");
        assert!(parsed.get("civ_pins").is_some(), "must have civ_pins");
        assert!(parsed.get("factions").is_some(), "must have factions");
        assert!(parsed.get("buildings").is_some(), "must have buildings");
        assert!(parsed.get("is_day").is_some(), "must have is_day");
    }
}
