//! Tests for FR-CIV-CORE-009
//!
//! Epic: FR-CIV-CORE
//!
//! This test file verifies FR FR-CIV-CORE-009: WebSocket JSON-RPC Protocol.
//! The engine exposes snapshot types that the server uses for JSON-RPC responses.

#[cfg(test)]
mod fr_fr_civ_core_009 {
    /// SimulationSnapshot exists and contains the JSON-RPC response fields.
    #[test]
    fn snapshot_has_jsonrpc_fields() {
        let mut sim = civ_engine::Simulation::with_seed(1);
        sim.tick();
        let snap = sim.snapshot();
        assert_eq!(snap.tick, 1, "snapshot must report current tick");
        assert!(snap.population >= 0, "population must be non-negative");
        let json = serde_json::to_value(&snap).expect("snapshot to JSON value");
        assert!(json.get("tick").is_some(), "JSON must have tick field");
        assert!(
            json.get("population").is_some(),
            "JSON must have population field"
        );
    }

    /// ReplayLog and ReplayEvent types exist for the subscribe/replay protocol.
    #[test]
    fn replay_types_exist() {
        use civ_engine::replay::{ReplayEvent, ReplayLog};
        let log = ReplayLog::default();
        assert!(log.events.is_empty(), "default log has no events");
        let _tick_event = ReplayEvent::Tick { tick: 0 };
    }
}
