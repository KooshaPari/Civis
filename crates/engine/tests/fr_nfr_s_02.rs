//! NFR-S-02 — engine-side baseline for the connection-overhead gate: the
//! initial world state that a handshake's first snapshot is built from starts
//! at tick 0 and is stable across fresh instances.
//!
//! The handshake latency gate itself (`ws_handshake_budget_met` /
//! `WS_HANDSHAKE_BUDGET_MS`) is covered by the unit tests in
//! `crates/server/src/perf_budgets.rs`.

#[cfg(test)]
mod fr_nfr_s_02 {
    use civ_engine::WorldState;

    // NFR-S-02 — fresh world state starts at tick 0 (handshake snapshot baseline).
    #[test]
    fn verify_nfr_s_02_basic() {
        let ws = WorldState::default();
        assert_eq!(ws.tick, 0);
        let ws2 = WorldState::default();
        assert_eq!(ws.tick, ws2.tick, "fresh instances must be stable");
    }
}
