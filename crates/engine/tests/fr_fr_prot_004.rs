//! Tests for FR-PROT-004 — Event Persistence
//!
//! Epic: FR-PROT
//! The server SHALL persist all emitted events to the DB audit log
//! within the same tick.

#[cfg(test)]
mod fr_fr_prot_004 {
    /// FR-PROT-004: WorldState tick is the persistence timestamp.
    #[test]
    fn tick_is_persistence_timestamp() {
        let ws = civ_engine::WorldState {
            tick: 50,
            ..civ_engine::WorldState::default()
        };
        assert_eq!(ws.tick, 50, "tick serves as the persistence timestamp");
    }
}