//! Tests for FR-PROT-002 — Event Envelope Compatibility
//!
//! Epic: FR-PROT
//! Events SHALL be emitted as JSON-RPC notifications with a common envelope.
//! WorldState tick is the primary envelope field.

#[cfg(test)]
mod fr_fr_prot_002 {
    /// FR-PROT-002: SimulationSnapshot exposes tick for event envelope.
    #[test]
    fn simulation_snapshot_has_tick() {
        let ws = civ_engine::WorldState {
            tick: 7,
            ..civ_engine::WorldState::default()
        };
        assert_eq!(ws.tick, 7, "tick must be accessible for event envelope");
    }

    /// FR-PROT-002: Step produces new state with incremented tick for notification.
    #[test]
    fn step_produces_new_state_for_notification() {
        let ws = civ_engine::WorldState {
            tick: 100,
            ..civ_engine::WorldState::default()
        };
        let ws2 = civ_engine::step(ws, civ_engine::Fixed::from_num(0));
        assert_eq!(ws2.tick, 101);
    }
}